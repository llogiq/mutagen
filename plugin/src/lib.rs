#![feature(proc_macro_span)] // for source_file / line / column display, nightly only

extern crate proc_macro;

use std::{borrow::Cow, collections::HashMap, fs::{create_dir_all, File, OpenOptions},
    hash::Hash, io::{BufWriter, Write}, mem, sync::atomic::{AtomicUsize, Ordering::SeqCst}};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{fold::Fold, spanned::Spanned, *};

mod mutation;
mod pattern;
mod ty;
use crate::mutation::MutationType;
use crate::pattern::{ArgTy, BindingMode, destructure_fn_arg, SelfOr};

macro_rules! bounds {
    { $($suf:ident, $ty:ident),* } => {
        fn int_boundaries(s: IntSuffix, negative: bool) -> &'static [u128] {
            match s {
                $(IntSuffix::$suf => {
                        &[$ty::min_value() as u128, $ty::max_value() as u128]
                }),*
                _ => {
                    if negative {
                        &[i128::min_value() as u128]
                    } else {
                        &[u128::max_value()]
                    }
                }
            }
        }
    }
}

bounds! {
    I8, i8, I16, i16, I32, i32, I64, i64, I128, i128, Isize, isize,
    U8, u8, U16, u16, U32, u32, U64, u64, U128, u128, Usize, usize
}

macro_rules! fold_binary {
    { $self:expr, $attrs:expr, $op: expr, $left:expr, $right:expr, $cov:expr;
        $($pat:pat => ( $op_trait:ty, $op_fn:tt, $op_found:tt, $op_repl:tt ))* ;
        $($pat2:pat => $tt:tt)*
    } => {
        match $op {
            $(
                $pat => {
                    let (left, right, cov) = ($left, $right, $cov);
                    let (n, flag, mask) = $self.mutations($op.span(),
                        &[(MutationType::OPORTUNISTIC_BINARY,
                            concat!("(opportunistically) replacing x ",
                                stringify!($op_found), " y with x ",
                                stringify!($op_repl), " y").into())]);
                    return parse_quote!(if false {
                            #left $op_found #right
                        } else {
                            mutagen::$op_trait::$op_fn(#left, #right, #n,
                                & #cov[#flag], #mask)
                        })
                }
            ),*
            $($pat2 => $tt),*
            op => {
                let (attrs, left, right) = ($attrs, $left, $right);
                Expr::Binary(ExprBinary { attrs, op, left, right })
            }
        }
    }
}

macro_rules! fold_assign_op {
    { $self:expr, $attrs:expr, $op: expr, $left:expr, $right:expr, $cov:expr;
        $($pat:pat => ( $op_trait:ty, $op_fn:tt, $op_found:tt, $op_repl:tt ))*
    } => {
        match $op {
            $(
                $pat => {
                    let (left, right, cov) = ($left, $right, $cov);
                    let (n, flag, mask) = $self.mutations($op.span(),
                        &[(MutationType::OPORTUNISTIC_UNARY,
                            concat!("(opportunistically) replacing x ",
                                stringify!($op_found), "= y with x ",
                                stringify!($op_repl), "= y").into())]);
                    return parse_quote!(::mutagen::$op_trait::$op_fn(
                        &mut #left, #right, #n, & #cov[#flag], #mask))
                }
            ),*
            op => {
                let (attrs, left, right) = ($attrs, $left, $right);
                Expr::AssignOp(ExprAssignOp { attrs, op, left, right })
            }
        }
    }
}

struct Resizer(usize);

impl Fold for Resizer {
    fn fold_lit_int(&mut self, i: LitInt) -> LitInt {
        LitInt::new(self.0 as u64, IntSuffix::Usize, i.span())
    }

    fn fold_expr_array(&mut self, a: ExprArray) -> ExprArray {
        let mut a = a;
        a.elems = std::iter::repeat::<Expr>(
                syn::parse_quote!(::std::sync::atomic::AtomicUsize::new(0)))
            .take(self.0).collect();
        a
    }
}

static TARGET_MUTAGEN: &'static str = "target/mutagen";
static MUTATIONS_LIST: &'static str = "mutations.txt";
static MUTATION_COUNT: AtomicUsize = AtomicUsize::new(0);

/// information about the current method
struct MethodInfo<'m> {
    /// which inputs have the same type as the output?
    have_output_type: Vec<SelfOr<'m, Ident>>,
    /// which inputs have the same type and could be switched?
    /// TODO refs vs. values
    interchangeables: HashMap<SelfOr<'m, Ident>, Vec<SelfOr<'m, Ident>>>,
    /// which inputs are mutable references
    ref_muts: Vec<SelfOr<'m, Ident>>,
}

/// our mutator
struct Mutagen {
    /// a file to write mutation info to
    mutations: BufWriter<File>,
    /// the current mutation number
    current_count: usize,
    /// the current coverage number
    coverage_count: usize,
    /// the mutation types currently in use
    types: MutationType,
    /// a list of mutation types we want to avoid
    restrictions: Vec<MutationType>,
    /// true if method has `&mut self`
    self_ref_mut: bool,
}

impl Mutagen {
    fn new(mutations: BufWriter<File>) -> Self {
        Mutagen {
            mutations,
            current_count: MUTATION_COUNT.load(SeqCst),
            coverage_count: 0,
            types: MutationType::empty(),
            restrictions: vec![],
            self_ref_mut: false,
        }
    }

    /// returns the parent's imposed restrictions
    fn parent_restrictions(&self) -> MutationType {
        let restriction_amount = self.restrictions.len();
        if restriction_amount < 2 {
            return MutationType::empty();
        }

        let current_index = restriction_amount - 2;
        self.restrictions
            .get(current_index)
            .cloned()
            .unwrap_or_else(MutationType::empty)
    }

    fn current_restrictions(&mut self) -> Option<&mut MutationType> {
        self.restrictions.last_mut()
    }

    fn set_restrictions(&mut self, types: MutationType) {
        if let Some(r) = self.current_restrictions() {
            *r = types;
        }
    }

    /// add a number of mutations while allowing for coverage reporting.
    ///
    /// The result is the mutation number, the coverage index and mask
    fn mutations(&mut self, span: Span, mutations: &[(MutationType, Cow<'static, str>)])
    -> (usize, usize, usize) {
        let avoid = self.parent_restrictions();
        let count = self.current_count;
        let nightly_span = span.unwrap();
        let source = nightly_span.source_file().path();
        let (start, end) = (nightly_span.start(), nightly_span.end());
        let span_desc = format!("{}@{}:{}-{}:{}", source.to_string_lossy(), start.line, start.column, end.line, end.column);
        for (i, &(ty, ref description)) in mutations.iter().enumerate() {
            // If the current mutation intersect with the mutation types to avoid, skip it and
            // keep iterating through the following mutations.
            if avoid.contains(ty) {
                continue;
            }

            // Record that mutation type has been added
            self.types.insert(ty);

            // Write current mutation to the file
            writeln!(self.mutations, "{} - {} - {} @ {}", count + i, description, ty.as_str(), span_desc).unwrap()
        }
        self.current_count += mutations.len();
        let (index, mask) = coverage(&mut self.coverage_count);
        (count, index, mask)
    }

    fn extract_method_sig_info<'m>(&self, sig: &'m MethodSig) -> Option<MethodInfo<'m>> {
        if sig.unsafety.is_some() {
            panic!("mutagen: unsafe code found");
        }
        if sig.constness.is_some() {
            None
        } else {
            Some(self.extract_method_info(&sig.decl))
        }
    }

    fn extract_method_info<'m>(&self, decl: &'m FnDecl) -> MethodInfo<'m> {
        let out_ty = match decl.output {
            ReturnType::Default => None,
            ReturnType::Type(_, ref ty) => Some(&*ty)
        };
        // arguments of output type
        let mut have_output_type = vec![];
        // add arguments of same type, so we can switch them?
        let mut argtypes: HashMap<SelfOr<Ident>, &ArgTy> = HashMap::new();
        let mut typeargs: HashMap<&ArgTy, Vec<SelfOr<Ident>>> = HashMap::new();
        let mut argdefs = vec![];
        let mut occs = vec![];
        let mut ref_muts = vec![];
        for (pos, arg) in decl.inputs.iter().enumerate() {
            destructure_fn_arg(&arg, &mut occs, pos, &mut argdefs);
        }
        for (sym, ty_args) in argdefs.iter() {
            if ty_args.3.is_empty() && out_ty.map_or(false,
                    |t| ty::self_or_ty_equal(&SelfOr::Other(t), &ty_args.1, decl.inputs.len() == 1)) {
                have_output_type.push(sym.clone());
            }
            if ty_args.0 == BindingMode::RefMut ||
                    ty_args.3.is_empty() && ty::is_ty_ref_mut(&ty_args.1) {
                ref_muts.push(sym.clone());
            }
            argtypes.insert(sym.clone(), ty_args);
            typeargs.entry(ty_args).or_insert_with(Vec::new).push(sym.clone());
        }
        let mut interchangeables = HashMap::new();
        for (_, symbols) in typeargs {
            if symbols.len() > 1 {
                combine(&mut interchangeables, &symbols);
            }
        }
        MethodInfo {
            have_output_type,
            interchangeables,
            ref_muts
        }
    }

    fn fold_outer_block(&mut self, method_info: MethodInfo, block: Block) -> Block {
        let mut block = block;
        let span = block.span();
        let outer_coverage_count = self.coverage_count;
        self.coverage_count = 0;
        let cov = syn::parse_str::<Ident>("_MUTAGEN_COVERAGE").unwrap_or_else(|_| panic!("can't create cov ident"));
        let self_ident = syn::parse_str::<PathSegment>("self").unwrap_or_else(|_| panic!("can't create self ident")).ident;
        // set up array for coverage recording and opportunistic return default
        let mut orig_stmts = mem::replace(&mut block.stmts, Vec::new());
        let (n, flag, mask) = self.mutations(span, &[
            (MutationType::RETURN_DEFAULT, "insert opportunistic return default()".into())]);
        let mut stmts : Vec<Stmt> = vec![Stmt::Expr(Expr::Verbatim(ExprVerbatim { tts: TokenStream::default() })),
            parse_quote!(if let Some(d) = ::mutagen::Defaulter::get_default(#n,
                    &#cov[#flag], #mask) { return d; })];

        // return args with output type
        for i in &method_info.have_output_type {
            let ident = if let SelfOr::Other(i) = i { i } else { &self_ident };
            let (n, flag, mask) = self.mutations(span, &[
                (MutationType::RETURN_ARGUMENT, format!("insert return {}", ident).into())]);
            stmts.push(parse_quote!(if ::mutagen::now(#n, & #cov[#flag], #mask) {
                                        return #ident;
                                    }));
        };
        // interchange arguments
        for (k, values) in &method_info.interchangeables {
            let key = if let SelfOr::Other(i) = k { i } else { &self_ident };
            for v in values {
                let value = if let SelfOr::Other(i) = v { i } else { &self_ident };
                let (n, flag, mask) = self.mutations(span, &[(MutationType::EXCHANGE_ARGUMENT,
                    format!("exchange {} with {}", key, value).into())]);
                stmts.push(parse_quote!(let (#key, #value) = if ::mutagen::now(#n,
                        &#cov[#flag], #mask) { (#value, #key)
                    } else {
                        (#key, #value)
                    };));
            }
        };
        // clone ref muts
        let (outer_mut_self, mut ref_mut_self) = (self.self_ref_mut, false);
        for i in &method_info.ref_muts {
            let ident = if let SelfOr::Other(i) = i { i } else { &self_ident };
            let (n, flag, mask) = self.mutations(span, &[(MutationType::CLONE_MUTABLE,
                format!("clone mutable reference {}", ident).into())]);
            let target_ident = if ident == "self" {
                ref_mut_self = true;
                syn::parse_str::<Ident>("__mutated_self").unwrap_or_else(|_| panic!("cannot mutate self"))
            } else {
                ident.clone()
            }; //TODO store in mutagen to fold paths
            self.self_ref_mut = ref_mut_self;
            let ident_clone = syn::parse_str::<Ident>(&format!("{}_clone", ident)).unwrap_or_else(|_| panic!("cannot create clone ident"));
            stmts.push(parse_quote!(let mut #ident_clone;));
            stmts.push(parse_quote!(let #target_ident = if ::mutagen::MayClone::may_clone(#ident) {
                  #ident_clone = ::mutagen::MayClone::clone(#ident, #n, &#cov[#flag], #mask);
                  &mut #ident_clone
              } else { #ident };));
        };
        // fold original statements
        stmts.extend(orig_stmts.drain(..).map(|stmt| self.fold_stmt(stmt)));
        // set correct coverage array length
        let cov_stmt = parse_quote!(
            static #cov : [::std::sync::atomic::AtomicUsize; 0] = [];);
        let mut resizer = Resizer(self.current_count);
        stmts[0] = resizer.fold_stmt(cov_stmt);
        block.stmts = stmts;
        // reinstate coverage count and self_ref_mut for nested functions
        self.coverage_count = outer_coverage_count;
        self.self_ref_mut = outer_mut_self;
        block
    }

    /// Je nach Möglichkeit +1 oder -1 addieren
    fn fold_lit_int(&mut self, lit: Lit, negative: bool) -> Expr {
        let cov = syn::parse_str::<Ident>("_MUTAGEN_COVERAGE").unwrap_or_else(|_| panic!("fold_lit_int: cov ident failed"));
        if let Lit::Int(ref literal) = lit {
            let span = lit.span();
            let val = literal.value() as u128; //TODO this may remove higher bits
            if int_boundaries(literal.suffix(), negative).iter().any(|x| *x == val) {
                if negative || val == 0 { // lower bound
                    let (n, flag, mask) = self.mutations(span,
                        &[(MutationType::ADD_ONE_TO_LITERAL, "increment literal by one".into())]);
                    return parse_quote!(mutagen::inc(#lit, #n, & #cov[#flag], #mask));
                } else { // upper bound
                    let (n, flag, mask) = self.mutations(span,
                        &[(MutationType::SUB_ONE_TO_LITERAL, "decrement literal by one".into())]);
                    return parse_quote!(mutagen::dec(#lit, #n, & #cov[#flag], #mask));
                }
            } else {
                let (n, flag, mask) = self.mutations(span,
                    &[(MutationType::ADD_ONE_TO_LITERAL, "increment literal by one".into()),
                      (MutationType::SUB_ONE_TO_LITERAL, "decrement literal by one".into())]);
                return parse_quote!(mutagen::inc_dec(#lit, #n, & #cov[#flag], #mask));
            }
        }
        Expr::Lit(ExprLit { attrs: Vec::new(), lit })
    }
}

/// combine the given `symbols` and add them to the interchangeables map
fn combine<S: Hash + Eq + Clone>(interchangeables: &mut HashMap<S, Vec<S>>, symbols: &[S]) {
    let symbol_amount = symbols.len();

    for (i, index) in symbols.iter().cloned().enumerate() {
        let change_with = (i + 1..symbol_amount).map(|i| symbols[i].clone()).collect();
        interchangeables.insert(index, change_with);
    }
}

fn coverage(coverage_count: &mut usize) -> (usize, usize) {
    let usize_bits = usize::max_value().count_ones() as usize;
    let usize_shift = usize_bits.trailing_zeros() as usize;
    let usize_mask = usize_bits - 1;
    let c = *coverage_count;
    *coverage_count += 1;
    (c >> usize_shift, 1 << (c & usize_mask))
}

impl Fold for Mutagen {
    fn fold_expr_unsafe(&mut self, _: syn::ExprUnsafe) -> syn::ExprUnsafe {
        panic!("mutagen: unsafe code found");
    }

    fn fold_item_fn(&mut self, i: ItemFn) -> ItemFn {
        if i.unsafety.is_some() {
            panic!("mutagen: unsafe code found");
        }
        if i.constness.is_some() {
            return i; // we cannot mutate const functions, so leave them alone
        }
        let method_info = self.extract_method_info(&i.decl);
        ItemFn {
            block: Box::new(self.fold_outer_block(method_info, *i.block)),
            ..i
        }
    }

    fn fold_impl_item_method(&mut self, i: ImplItemMethod) -> ImplItemMethod {
        if let Some(method_info) = self.extract_method_sig_info(&i.sig) {
            ImplItemMethod {
                block: self.fold_outer_block(method_info, i.block),
                ..i
            }
        } else {
            i
        }
    }

    fn fold_trait_item_method(&mut self, i: TraitItemMethod) -> TraitItemMethod {
        let mut i = i;
        let default = mem::replace(&mut i.default, None);
        if let (Some(block), Some(method_info)) =
                (default, self.extract_method_sig_info(&i.sig)) {
            TraitItemMethod {
                default: Some(self.fold_outer_block(method_info, block)),
                ..i
            }
        } else {
            i
        }
    }

    fn fold_expr(&mut self, e: Expr) -> Expr {
        let cov = &syn::parse_str::<Ident>("_MUTAGEN_COVERAGE").unwrap_or_else(|_| panic!("fold_expr: cov ident failed"));
        let span = e.span();

        match e {
            Expr::AssignOp(ExprAssignOp { attrs, left, op, right }) => {
                fold_assign_op! { self, attrs, op, left, right, cov;
                    BinOp::Add(_) => ( AddSub, add_sub, +, - )
                    BinOp::Sub(_) => ( SubAdd, sub_add, -, + )
                    BinOp::Mul(_) => ( MulDiv, mul_div, *, / )
                    BinOp::Div(_) => ( DivMul, div_mul, /, * )
                    BinOp::Shl(_) => ( ShlShr, shl_shr, <<, >> )
                    BinOp::Shr(_) => ( ShrShl, shr_shl, >>, << )
                    BinOp::BitAnd(_) => ( BitAndBitOr, bitand_bitor, &, | )
                    BinOp::BitOr(_) => ( BitOrBitAnd, bitor_bitand, |, & )
                }
            }
            Expr::Binary(ExprBinary { attrs, left, op, right }) => {
                let left = Box::new(self.fold_expr(*left));
                let right = Box::new(self.fold_expr(*right));
                fold_binary! { self, attrs, op, left, right, cov;
                    BinOp::Add(_) => ( AddSub, add_sub, +, - )
                    BinOp::Sub(_) => ( SubAdd, sub_add, -, + )
                    BinOp::Mul(_) => ( MulDiv, mul_div, *, / )
                    BinOp::Div(_) => ( DivMul, div_mul, /, * )
                    BinOp::Shl(_) => ( ShlShr, shl_shr, <<, >> )
                    BinOp::Shr(_) => ( ShrShl, shr_shl, >>, << )
                    BinOp::BitAnd(_) => ( BitAndBitOr, bitand_bitor, &, | )
                    BinOp::BitOr(_) => ( BitOrBitAnd, bitor_bitand, |, & );
                    BinOp::And(_) => {
                        // avoid restrictions that would lead to a false evaluation, we will already replace
                        // the and with a false expression
                        self.set_restrictions(MutationType::REPLACE_WITH_FALSE);
                        let (n, flag, mask) = self.mutations(
                                span,
                                &[
                                    (MutationType::REPLACE_WITH_FALSE, "replacing _ && _ with false".into()),
                                    (MutationType::REPLACE_WITH_TRUE, "replacing _ && _ with true".into()),
                                    (MutationType::REMOVE_RIGHT, "replacing x && _ with x".into()),
                                    (MutationType::NEGATE_LEFT, "replacing x && _ with !x".into()),
                                    (MutationType::NEGATE_RIGHT, "replacing x && y with x && !y".into()),
                                ],
                            );
                        parse_quote!(match (#left, ::mutagen::diff(#n, 5, & #cov[#flag], #mask)) {
                            (_, 0) => false,
                            (_, 1) => true,
                            (x, 2) => x,
                            (x, 3) => !x,
                            (x, n) => x && (#right) == (n != 4),
                        })
                    }
                    BinOp::Or(_) => {
                        self.set_restrictions(MutationType::REPLACE_WITH_TRUE);

                        let (n, flag, mask) = self.mutations(span, &[
                            (MutationType::REPLACE_WITH_FALSE, "replacing _ || _ with false".into()),
                            (MutationType::REPLACE_WITH_TRUE, "replacing _ || _ with true".into()),
                            (MutationType::REMOVE_RIGHT, "replacing x || _ with x".into()),
                            (MutationType::NEGATE_LEFT, "replacing x || _ with !x".into()),
                            (MutationType::NEGATE_RIGHT, "replacing x || y with x || !y".into()),
                        ]);
                        parse_quote!(match (#left, ::mutagen::diff(#n, 5, & #cov[#flag], #mask)) {
                            (_, 0) => false,
                            (_, 1) => true,
                            (x, 2) => x,
                            (x, 3) => !x,
                            (x, n) => x || (#right) == (n != 4),
                        })
                    }
                    BinOp::Eq(_) => {
                        let (n, flag, mask) = self.mutations(span, &[
                            (MutationType::REPLACE_WITH_TRUE, "replacing _ == _ with true".into()),
                            (MutationType::REPLACE_WITH_FALSE, "replacing _ == _ with false".into()),
                            (MutationType::NEGATE_EXPRESSION, "replacing x == y with x != y".into()),
                        ]);
                        parse_quote!(::mutagen::eq(&#left, &#right, #n, & #cov[#flag], #mask))
                    }
                    BinOp::Ne(_) => {
                        let (n, flag, mask) = self.mutations(span, &[
                            (MutationType::REPLACE_WITH_TRUE, "replacing _ != _ with true".into()),
                            (MutationType::REPLACE_WITH_FALSE, "replacing _ != _ with false".into()),
                            (MutationType::NEGATE_EXPRESSION, "replacing x != y with x == y".into()),
                        ]);
                        parse_quote!(::mutagen::ne(&#left, &#right, #n, & #cov[#flag], #mask))
                    }
                    BinOp::Gt(_) => {
                        let (n, flag, mask) = self.mutations(span, &[
                            (MutationType::REPLACE_WITH_FALSE, "replacing _ > _ with false".into()),
                            (MutationType::REPLACE_WITH_TRUE, "replacing _ > _ with true".into()),
                            (MutationType::COMPARISON, "replacing x > y with x < y".into()),
                            (MutationType::NEGATE_EXPRESSION, "replacing x > y with x <= y".into()),
                            (MutationType::COMPARISON, "replacing x > y with x >= y".into()),
                            (MutationType::COMPARISON, "replacing x > y with x == y".into()),
                            (MutationType::COMPARISON, "replacing x > y with x != y".into()),
                        ]);
                        parse_quote!(::mutagen::gt(&#left, &#right, #n, & #cov[#flag], #mask))
                    }
                    BinOp::Lt(_) => {
                        let (n, flag, mask) = self.mutations(span, &[
                            (MutationType::REPLACE_WITH_FALSE, "replacing _ < _ with false".into()),
                            (MutationType::REPLACE_WITH_TRUE, "replacing _ < _ with true".into()),
                            (MutationType::COMPARISON, "replacing x < y with x > y".into()),
                            (MutationType::NEGATE_EXPRESSION, "replacing x < y with x >= y".into()),
                            (MutationType::COMPARISON, "replacing x < y with x <= y".into()),
                            (MutationType::COMPARISON, "replacing x < y with x == y".into()),
                            (MutationType::COMPARISON, "replacing x < y with x != y".into()),
                        ]);
                        parse_quote!(::mutagen::lt(&#left, &#right, #n, & #cov[#flag], #mask))
                    }
                    BinOp::Ge(_) => {
                        let (n, flag, mask) = self.mutations(span, &[
                            (MutationType::REPLACE_WITH_FALSE, "replacing _ >= _ with false".into()),
                            (MutationType::REPLACE_WITH_TRUE, "replacing _ >= _ with true".into()),
                            (MutationType::NEGATE_EXPRESSION, "replacing x >= y with x < y".into()),
                            (MutationType::COMPARISON, "replacing x >= y with x <= y".into()),
                            (MutationType::COMPARISON, "replacing x >= y with x > y".into()),
                            (MutationType::COMPARISON, "replacing x >= y with x == y".into()),
                            (MutationType::COMPARISON, "replacing x >= y with x != y".into()),
                        ]);
                        parse_quote!(::mutagen::ge(&#left, &#right, #n, & #cov[#flag], #mask))
                    }
                    BinOp::Le(_) => {
                        let (n, flag, mask) = self.mutations(span, &[
                            (MutationType::REPLACE_WITH_FALSE, "replacing _ <= _ with false".into()),
                            (MutationType::REPLACE_WITH_TRUE, "replacing _ <= _ with true".into()),
                            (MutationType::NEGATE_EXPRESSION, "replacing x <= y with x > y".into()),
                            (MutationType::COMPARISON, "replacing x <= y with x >= y".into()),
                            (MutationType::COMPARISON, "replacing x <= y with x < y".into()),
                            (MutationType::COMPARISON, "replacing x <= y with x == y".into()),
                            (MutationType::COMPARISON, "replacing x <= y with x != y".into()),
                        ]);
                        parse_quote!(::mutagen::le(&#left, &#right, #n, & #cov[#flag], #mask))
                    }
                }
            }
            Expr::Unary(ExprUnary { attrs, op, expr }) => {
            //    pub op: UnOp,
            //    pub expr: Box<Expr>,
                match op {
                    UnOp::Neg(_) => {
                        if let Expr::Lit(ExprLit { lit, .. }) = *expr {
                            self.fold_lit_int(lit, true)
                        } else {
                            let expr = self.fold_expr(*expr);
                            let (n, flag, mask) = self.mutations(span,
                                &[(MutationType::OPORTUNISTIC_UNARY, "(opportunistically) removing -".into())]);
                            parse_quote!(::mutagen::MayNeg::may_neg(&#expr, #n, &#cov[#flag], #mask))
                        }
                    }
                    UnOp::Not(_) => {
                        let expr = self.fold_expr(*expr);
                        let (n, flag, mask) = self.mutations(span,
                            &[(MutationType::OPORTUNISTIC_UNARY, "(opportunistically) removing !".into())]);
                        parse_quote!(::mutagen::MayNot::may_not(&#expr, #n, &#cov[#flag], #mask))
                    }
                    UnOp::Deref(_) => {
                        let expr = Box::new(self.fold_expr(*expr));
                        Expr::Unary(ExprUnary { attrs, op, expr })
                    }
                }
            }
            Expr::Lit(ExprLit { lit, .. }) => {
                self.fold_lit_int(lit, false)
            }
            Expr::If(ExprIf { attrs, if_token, cond, then_branch, mut else_branch }) => {
                self.set_restrictions(
                    MutationType::REPLACE_WITH_TRUE |
                    MutationType::REPLACE_WITH_FALSE |
                    MutationType::NEGATE_EXPRESSION
                );
                let cond = Box::new(self.fold_expr(*cond));
                let then_branch = self.fold_block(then_branch);
                if let Some((else_token, else_expr)) = else_branch {
                    else_branch = Some((else_token, Box::new(self.fold_expr(*else_expr))));
                }
                let (n, flag, mask) = self.mutations(
                    cond.span(),
                    &[
                        (MutationType::REPLACE_WITH_TRUE, "replacing if condition with true".into()),
                        (MutationType::REPLACE_WITH_FALSE, "replacing if condition with false".into()),
                        (MutationType::NEGATE_EXPRESSION, "inverting if condition".into()),
                    ],
                );
                let cond = parse_quote!(::mutagen::t(#cond, #n, & #cov[#flag], #mask));
                Expr::If(ExprIf { attrs, if_token, cond, then_branch, else_branch })
            }
            Expr::While(ExprWhile { attrs, label, while_token, cond, body }) => {
                let (n, flag, mask) = self.mutations(
                        cond.span(),
                        &[(MutationType::REPLACE_WITH_FALSE, "replacing while condition with false".into())],
                    );
                let cond = self.fold_expr(*cond);
                let body = self.fold_block(body);
                let cond = Box::new(parse_quote!(::mutagen::w(#cond, #n, &#cov[#flag], #mask)));
                Expr::While(ExprWhile { attrs, label, while_token, cond, body })
            }
            Expr::ForLoop(ExprForLoop { attrs, label, for_token, pat, in_token, expr, body }) => {
                let (n, flag, mask) = self.mutations(
                    expr.span(),
                    &[
                        (MutationType::ITERATOR_EMPTY, "empty iterator".into()),
                        (MutationType::ITERATOR_SKIP_FIRST, "skip first element".into()),
                        (MutationType::ITERATOR_SKIP_LAST, "skip last element".into()),
                        (MutationType::ITERATOR_SKIP_BOUNDS, "skip first and last element".into()),
                    ],
                );
                let pat = Box::new(self.fold_pat(*pat));
                let expr = Box::new(self.fold_expr(*expr));
                let body = self.fold_block(body);
                let expr = parse_quote!(::mutagen::forloop(#expr, #n, &#cov[#flag], #mask));
                Expr::ForLoop(ExprForLoop { attrs, label, for_token, pat, in_token, expr, body })
            }
            e => fold::fold_expr(self, e)
        }
    }
}

#[proc_macro_attribute]
pub fn mutate(_attrs: proc_macro::TokenStream, code: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(code as Item);
    // create target/mutagen path if it doesn't exist
    let mutagen_dir = std::path::Path::new(TARGET_MUTAGEN);
    if !mutagen_dir.exists() {
        create_dir_all(&mutagen_dir).unwrap();
    }
    let mutation_fpath = mutagen_dir.join(MUTATIONS_LIST);
    let mutation_file = if MUTATION_COUNT.compare_and_swap(0, 1, SeqCst) > 0 {
        OpenOptions::new().append(true).open(mutation_fpath)
    } else {
        File::create(mutation_fpath)
    }.unwrap();
    let mutations = BufWriter::new(mutation_file);
    let mut mutagen = Mutagen::new(mutations);
    let item = mutagen.fold_item(input);
    MUTATION_COUNT.store(mutagen.current_count, SeqCst);
    proc_macro::TokenStream::from(quote!(#item))
}
