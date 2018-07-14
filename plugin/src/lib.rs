#![feature(plugin_registrar, quote, rustc_private, custom_attribute)]

extern crate rustc_plugin;
extern crate syntax;
extern crate mutagen;
#[macro_use]
extern crate bitflags;

use rustc_plugin::registry::Registry;
use std::collections::HashMap;
use std::fs::{create_dir_all, File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::{BufWriter, Write};
use std::iter::repeat;
use std::mem;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use syntax::ast::*;
use syntax::codemap::{Span, Spanned};
use syntax::ext::base::{Annotatable, ExtCtxt, SyntaxExtension};
use syntax::fold::{self, Folder};
use syntax::ptr::P;
use syntax::symbol::Symbol;
use syntax::util::small_vector::SmallVector;
use syntax::ast::{IntTy, LitIntType, LitKind, UnOp};
use syntax::ext::base::MultiItemModifier;

mod binop {
    include!(concat!(env!("OUT_DIR"), "/plugin_ops.rs"));
}
mod bounded_loop;


/// ChainedMultiMutator is a MultiMutator which allows to chain two `MultiItemModifier`.
struct ChainedMultiMutator {
    left: Box<MultiItemModifier>,
    right: Box<MultiItemModifier>,
}

impl MultiItemModifier for ChainedMultiMutator {
    fn expand(&self, cx: &mut ExtCtxt, span: Span, mi: &MetaItem, a: Annotatable) -> Vec<Annotatable> {
        let out = self.left.expand(cx, span, mi, a);

        out.into_iter()
            .map(|a| self.right.expand(cx, span, mi, a))
            .collect::<Vec<Vec<Annotatable>>>()
            .into_iter()
            .fold(Vec::new(), |mut acc, outs| {
                acc.extend(outs);

                acc
            })
    }
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    let chained = Box::new(ChainedMultiMutator {
        left: Box::new(mutator),
        right: Box::new(bounded_loop::bounded_loop),
    });

    reg.register_syntax_extension(
        Symbol::intern("mutate"),
        SyntaxExtension::MultiModifier(chained),
    );
}

static TARGET_MUTAGEN: &'static str = "target/mutagen";
static MUTATIONS_LIST: &'static str = "mutations.txt";
static MUTATION_COUNT: AtomicUsize = AtomicUsize::new(0);

/// create a MutatorPlugin and let it fold the items/trait items/impl items
pub fn mutator(cx: &mut ExtCtxt, _span: Span, _mi: &MetaItem, a: Annotatable) -> Annotatable {
    // create target/mutagen path if it doesn't exist
    let mutagen_dir = if cx.root_path.ends_with("src") {
        cx.root_path.parent().unwrap_or_else(|| ::std::path::Path::new("."))
    } else {
        cx.root_path.as_path()
    }.join(TARGET_MUTAGEN);
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
    let mut p = MutatorPlugin::new(cx, mutations, MUTATION_COUNT.load(SeqCst));
    let result = match a {
        Annotatable::Item(i) => {
            Annotatable::Item(p.fold_item(i).expect_one("expected exactly one item"))
        }
        Annotatable::TraitItem(i) => Annotatable::TraitItem(i.map(|i| {
            p.fold_trait_item(i).expect_one("expected exactly one item")
        })),
        Annotatable::ImplItem(i) => Annotatable::ImplItem(i.map(|i| {
            p.fold_impl_item(i).expect_one("expected exactly one item")
        })),
        stmt_or_expr => stmt_or_expr,
    };
    p.m.mutations.flush().unwrap();
    MUTATION_COUNT.store(p.m.current_count, SeqCst);
    result
}

/// information about the current method
struct MethodInfo {
    /// which inputs have the same type as the output?
    have_output_type: Vec<Symbol>,
    /// which inputs have the same type and could be switched?
    /// TODO refs vs. values
    interchangeables: HashMap<Symbol, Vec<Symbol>>,
    /// which inputs are mutable references
    ref_muts: Vec<Symbol>,
    /// the generated symbol for coverage
    coverage_sym: Symbol,
    /// the count of coverage calls
    coverage_count: usize,
    /// a symbol to store `self` in
    self_sym: Option<Symbol>
}

#[derive(Default)]
struct MutatorInfo {
    /// a stack of method infos
    method_infos: Vec<MethodInfo>,
    /// Self types for known impls
    self_tys: Vec<Ty>,
}

struct Mutator<'a, 'cx: 'a> {
    /// context for quoting
    cx: &'a mut ExtCtxt<'cx>,
    /// a sequence of mutations
    mutations: BufWriter<File>,
    /// the current mutation count, starting from 1
    current_count: usize,
    /// set of the types that have been generated
    types: MutationType,
}

impl<'a, 'cx: 'a> Mutator<'a, 'cx> {
    fn add_mutations<'m>(&mut self, span: Span, avoid: MutationType, descriptions: &[Mutation<'m>]) -> usize {
        let initial_count = self.current_count;
        let span_desc = self.cx.codemap().span_to_string(span);
        for (i, mutation) in descriptions.iter().enumerate() {
            // If the current mutation intersect with the mutation types to avoid, skip it and
            // keep iterating through the following mutations.
            if avoid.contains(mutation.ty) {
                continue;
            }

            // Record that mutation type has been added
            self.types.insert(mutation.ty);

            // Write current mutation to the file
            writeln!(&mut self.mutations, "{} - {} - {} @ {}", initial_count + i, mutation.description, mutation.ty.to_string(), span_desc).unwrap()
        }
        self.current_count += descriptions.len();
        initial_count
    }
}

struct Mutation<'a> {
    ty: MutationType,
    description: &'a str,
}

impl<'a> Mutation<'a> {
    pub fn new(ty: MutationType, description: &'a str) -> Self {
        Mutation {
            ty,
            description,
        }
    }
}

bitflags! {
    struct MutationType: u64 {
        const OTHER = 1 << 0;
        const REPLACE_WITH_TRUE = 1 << 1;
        const REPLACE_WITH_FALSE = 1 << 2;
        const ADD_ONE_TO_LITERAL = 1 << 3;
        const SUB_ONE_TO_LITERAL = 1 << 4;
        const REMOVE_RIGHT = 1 << 5;
        const NEGATE_LEFT = 1 << 6;
        const NEGATE_RIGHT = 1 << 7;
        const NEGATE_EXPRESSION = 1 << 8;
        const COMPARISON = 1 << 9;
        const OPORTUNISTIC_BINARY = 1 << 10;
        const OPORTUNISTIC_UNARY = 1 << 11;
        const ITERATOR_EMPTY = 1 << 12;
        const ITERATOR_SKIP_FIRST = 1 << 13;
        const ITERATOR_SKIP_LAST = 1 << 14;
        const ITERATOR_SKIP_BOUNDS = 1 << 15;
        const RETURN_DEFAULT = 1 << 16;
        const RETURN_ARGUMENT = 1 << 17;
        const EXCHANGE_ARGUMENT = 1 << 18;
        const CLONE_MUTABLE = 1 << 19;
    }
}

impl ToString for MutationType {
    fn to_string(&self) -> String {
        match *self {
            MutationType::OTHER => String::from("OTHER"),
            MutationType::REPLACE_WITH_TRUE => String::from("REPLACE_WITH_TRUE"),
            MutationType::REPLACE_WITH_FALSE => String::from("REPLACE_WITH_FALSE"),
            MutationType::ADD_ONE_TO_LITERAL => String::from("ADD_ONE_TO_LITERAL"),
            MutationType::SUB_ONE_TO_LITERAL => String::from("SUB_ONE_TO_LITERAL"),
            MutationType::REMOVE_RIGHT => String::from("REMOVE_RIGHT"),
            MutationType::NEGATE_LEFT => String::from("NEGATE_LEFT"),
            MutationType::NEGATE_RIGHT => String::from("NEGATE_RIGHT"),
            MutationType::NEGATE_EXPRESSION => String::from("NEGATE_EXPRESSION"),
            MutationType::COMPARISON => String::from("COMPARISION"),
            MutationType::OPORTUNISTIC_BINARY => String::from("OPORTUNISTIC_BINARY"),
            MutationType::OPORTUNISTIC_UNARY => String::from("OPORTUNISTIC_UNARY"),
            MutationType::ITERATOR_EMPTY => String::from("ITERATOR_EMPTY"),
            MutationType::ITERATOR_SKIP_FIRST => String::from("ITERATOR_SKIP_FIRST"),
            MutationType::ITERATOR_SKIP_LAST => String::from("ITERATOR_SKIP_LAST"),
            MutationType::ITERATOR_SKIP_BOUNDS => String::from("ITERATOR_SKIP_BOUNDS"),
            MutationType::RETURN_DEFAULT=> String::from("RETURN_DEFAULT"),
            MutationType::RETURN_ARGUMENT => String::from("RETURN_ARGUMENT"),
            MutationType::EXCHANGE_ARGUMENT => String::from("EXCHANGE_ARGUMENT"),
            MutationType::CLONE_MUTABLE => String::from("CLONE_MUTABLE"),
            _ => String::from("UNKNOWN"),
        }
    }
}

/// The MutatorPlugin
pub struct MutatorPlugin<'a, 'cx: 'a> {
    /// information about the context
    info: MutatorInfo,
    /// the mutator itself
    m: Mutator<'a, 'cx>,
    /// stack of mutation restrictions
    restrictions: Vec<MutationType>,
}

struct Resizer(usize);

impl Folder for Resizer {
    fn fold_expr(&mut self, expr: P<Expr>) -> P<Expr> {
        expr.map(|expr| {
            match expr {
                Expr { id, node: ExprKind::Lit(lit), span, attrs } => {
                    Expr {
                        id,
                        node: ExprKind::Lit(lit.map(|Spanned { span, .. }|
                            Spanned { span,
                                node: LitKind::Int(self.0 as u128, LitIntType::Unsigned(UintTy::Usize)) }
                        )),
                        span,
                        attrs,
                    }
                }
                Expr { id, node: ExprKind::Repeat(elem, _), span, attrs } => {
                    Expr {
                        id,
                        node: ExprKind::Array(repeat(elem).take(self.0).collect()),
                        span,
                        attrs,
                    }
                }
                expr => fold::noop_fold_expr(expr, self)
            }
        })
    }
}


/// a combination of BindingMode, type and occurrence within the type
#[derive(Clone, Eq, Debug)]
struct ArgTy<'t>(BindingMode, &'t Ty, usize, Vec<TyOcc>);

impl<'t> PartialEq for ArgTy<'t> {
    fn eq(&self, other: &ArgTy<'t>) -> bool {
        self.0 == other.0 && ty_equal(self.1, other.1, self.2 == other.2) && self.3 == other.3
    }
}

impl<'t> Hash for ArgTy<'t> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        ty_hash(self.1, self.2, state);
        self.3.hash(state);
    }
}

impl<'a, 'cx> MutatorPlugin<'a, 'cx> {
    fn new(cx: &'a mut ExtCtxt<'cx>, mutations: BufWriter<File>, count: usize) -> Self {
        MutatorPlugin {
            info: Default::default(),
            m: Mutator {
                cx,
                mutations,
                current_count: count,
                types: MutationType::empty(),
            },
            restrictions: Vec::new(),
        }
    }

    fn add_mutations<'m>(&mut self, span: Span, mutations: &[Mutation<'m>]) -> (usize, Ident, usize, usize) {
        let avoid = self.parent_restrictions();
        let start_count = self.m.add_mutations(span, avoid, mutations);
        let info = self.info.method_infos.last_mut().unwrap();
        // must be in a method
        let sym = Ident::with_empty_ctxt(info.coverage_sym);
        let (flag, mask) = coverage(&mut info.coverage_count);
        (start_count, sym, flag, mask)
    }

    fn cx(&mut self) -> &mut ExtCtxt<'cx> {
        self.m.cx
    }

    fn start_fn(&mut self, decl: &FnDecl) {
        let out_ty = match decl.output {
            FunctionRetTy::Default(_) => None,
            FunctionRetTy::Ty(ref ty) => Some(&**ty)
        };
        // arguments of output type
        let mut have_output_type = vec![];
        // add arguments of same type, so we can switch them?
        let mut argtypes: HashMap<Symbol, ArgTy> = HashMap::new();
        let mut typeargs: HashMap<ArgTy, Vec<Symbol>> = HashMap::new();
        let mut argdefs = vec![];
        let mut occs = vec![];
        let mut ref_muts = vec![];
        let mut self_sym = None;
        for (pos, arg) in decl.inputs.iter().enumerate() {
            destructure_bindings(&arg.pat, &*arg.ty, &mut occs, pos, &mut argdefs);
            if self_sym.is_none() && arg.is_self() {
                self_sym = Some(Symbol::gensym("__self_mutated"));
            }
        }
        for (sym, ty_args) in argdefs {
            if ty_args.3.is_empty() && out_ty.map_or(false, |t| ty_equal(t, ty_args.1, decl.inputs.len() == 1)) {
                have_output_type.push(sym);
            }
            if ty_args.0 == BindingMode::ByRef(Mutability::Mutable) ||
                    ty_args.3.is_empty() && is_ty_ref_mut(&ty_args.1) {
                ref_muts.push(sym);
            }
            argtypes.insert(sym, ty_args.clone());
            typeargs.entry(ty_args).or_insert_with(Vec::new).push(sym);
        }

        let mut interchangeables = HashMap::new();
        for (_, symbols) in typeargs {
            if symbols.len() > 1 {
                combine(&mut interchangeables, &symbols);
            }
        }
        let coverage_sym = Symbol::gensym(&format!("__COVERAGE{}", self.m.current_count));
        self.info.method_infos.push(MethodInfo {
            have_output_type,
            interchangeables,
            ref_muts,
            coverage_sym,
            coverage_count: 0,
            self_sym,
        });
    }

    fn end_fn(&mut self) {
        let info = self.info.method_infos.pop();
        assert!(info.is_some());
    }

    fn start_impl(&mut self, ty: &Ty) {
        self.info.self_tys.push(ty.clone());
    }

    fn end_impl(&mut self) {
        let ty = self.info.self_tys.pop();
        assert!(ty.is_some());
    }

    fn get_self_sym(&self) -> Option<Symbol> {
        self.info.method_infos.last().and_then(|info| info.self_sym)
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

    fn mutate_numeric_constant_expression(
        &mut self,
        lit: &Lit,
        is_negative: bool,
    ) -> Option<P<Expr>> {
        match *lit {
            Spanned {
                node: LitKind::Int(i, ty),
                span: s,
            } => {
                let mut mut_expression = quote_expr!(self.cx(), $lit);

                let mut numeric_constant = i as i128;
                if is_negative {
                    numeric_constant = -numeric_constant;
                }

                if int_constant_can_subtract_one(numeric_constant, ty) {
                    let (n, sym, flag, mask) = self.add_mutations(
                            s,
                            &[Mutation::new(MutationType::SUB_ONE_TO_LITERAL, "sub one from int constant")],
                        );
                    mut_expression = quote_expr!(self.cx(),
                                    {
                                        if ::mutagen::now($n, &$sym[$flag], $mask) { $lit - 1 }
                                        else { $mut_expression }
                                    });
                }

                if int_constant_can_add_one(numeric_constant as u128, ty) {
                    let (n, sym, flag, mask) = self.add_mutations(
                            s,
                            &[Mutation::new(MutationType::ADD_ONE_TO_LITERAL, "add one to int constant")],
                        );
                    mut_expression = quote_expr!(self.cx(),
                                    {
                                        if ::mutagen::now($n, &$sym[$flag], $mask) { $lit + 1 }
                                        else { $mut_expression }
                                    });
                }

                Some(mut_expression)
            }
            _ => None,
        }
    }
}

impl<'a, 'cx> Folder for MutatorPlugin<'a, 'cx> {
    fn fold_impl_item(&mut self, i: ImplItem) -> SmallVector<ImplItem> {
        SmallVector::one(match i {
            ImplItem {
                id,
                ident,
                vis,
                defaultness,
                attrs,
                generics,
                node: ImplItemKind::Method(sig, block),
                span,
                tokens,
            } => {
                self.start_fn(&sig.decl);
                let ii = ImplItem {
                    id,
                    ident,
                    vis,
                    defaultness,
                    attrs,
                    generics,
                    node: ImplItemKind::Method(sig, fold_first_block(block, self)),
                    span,
                    tokens,
                };
                self.end_fn();

                ii
            },
            ii => ii,
        })
    }

    fn fold_trait_item(&mut self, i: TraitItem) -> SmallVector<TraitItem> {
        SmallVector::one(match i {
            TraitItem {
                id,
                ident,
                attrs,
                generics,
                node: TraitItemKind::Method(sig, Some(block)),
                span,
                tokens,
            } => {
                self.start_fn(&sig.decl);
                let ti = TraitItem {
                    id,
                    ident,
                    attrs,
                    generics,
                    node: TraitItemKind::Method(sig, Some(fold_first_block(block, self))),
                    span,
                    tokens,
                };
                self.end_fn();
                ti
            }
            ti => ti,
        })
    }

    fn fold_item_kind(&mut self, i: ItemKind) -> ItemKind {
        match i {
            ItemKind::Impl(
                unsafety,
                polarity,
                defaultness,
                generics,
                opt_trait_ref,
                ty,
                impl_items,
            ) => {
                if unsafety == Unsafety::Unsafe {
                    self.cx().parse_sess.span_diagnostic.fatal("mutagen: unsafe code found")
                                                        .raise();
                }
                self.start_impl(&ty);
                let k = ItemKind::Impl(
                    unsafety,
                    polarity,
                    defaultness,
                    generics,
                    opt_trait_ref,
                    ty,
                    impl_items.into_iter().flat_map(|ii| self.fold_impl_item(ii).into_iter()).collect(),
                );
                self.end_impl();
                k
            }
            ItemKind::Fn(decl, header, generics, block) => {
                if header.unsafety == Unsafety::Unsafe {
                    self.cx().parse_sess.span_diagnostic.fatal("mutagen: unsafe code found")
                                                        .raise();
                }
                self.start_fn(&decl);
                let k = ItemKind::Fn(
                    decl,
                    header,
                    generics,
                    fold_first_block(block, self),
                );
                self.end_fn();
                k
            }
            s @ ItemKind::Static(..) | s @ ItemKind::Const(..) => s,
            k => fold::noop_fold_item_kind(k, self),
        }
    }

    fn fold_block(&mut self, block: P<Block>) -> P<Block> {
        if block.rules != BlockCheckMode::Default {
            self.cx().parse_sess.span_diagnostic.fatal("mutagen: unsafe code found")
                                                .raise();
        }
        fold::noop_fold_block(block, self)
    }

    fn fold_expr(&mut self, expr: P<Expr>) -> P<Expr> {
        self.restrictions.push(MutationType::empty());

        let e = expr.and_then(|expr| match expr {
            e @ Expr {
                node: ExprKind::Mac(_),
                ..
            } => {
                // self.cx.expander().fold_expr(P(e)).map(|e| fold::noop_fold_expr(e, self))
                // ignore macros for now
                P(e)
            }
            Expr {
                node: ExprKind::Paren(e),
                ..
            } => {
                // This expression kind is just for pretty print. Inherit the restrictions from
                // parent for descendant expressions.
                let parent = self.parent_restrictions();
                self.set_restrictions(parent);
                self.fold_expr(e)
            }
            Expr {
                id,
                node: ExprKind::Binary(op, left, right),
                span,
                attrs,
            } => {
                binop::fold_binop(self, id, op, left, right, span, attrs)
            }
            Expr {
                id,
                node: ExprKind::AssignOp(op, left, right),
                span,
                attrs,
            } => {
                let left = self.fold_expr(left);
                let right = self.fold_expr(right);
                binop::fold_assignop(self, id, op, left, right, span, attrs)
            }
            Expr {
                id,
                node: ExprKind::If(cond, then, opt_else),
                span,
                attrs,
            } => {
                let cond_span = cond.span;
                self.set_restrictions(
                    MutationType::REPLACE_WITH_TRUE |
                    MutationType::REPLACE_WITH_FALSE |
                    MutationType::NEGATE_EXPRESSION
                );
                let cond = self.fold_expr(cond);

                let (n, sym, flag, mask) = self.add_mutations(
                    cond_span,
                    &[
                        Mutation::new(MutationType::REPLACE_WITH_TRUE, "replacing if condition with true"),
                        Mutation::new(MutationType::REPLACE_WITH_FALSE, "replacing if condition with false"),
                        Mutation::new(MutationType::NEGATE_EXPRESSION, "inverting if condition"),
                    ],
                );
                let then = fold::noop_fold_block(then, self);
                let opt_else = opt_else.map(|p_else| self.fold_expr(p_else));
                let mut_cond = quote_expr!(self.cx(), ::mutagen::t($cond, $n, &$sym[$flag], $mask));
                P(Expr {
                    id,
                    node: ExprKind::If(mut_cond, then, opt_else),
                    span,
                    attrs,
                })
            }
            Expr {
                id,
                node: ExprKind::While(cond, block, opt_label),
                span,
                attrs,
            } => {
                let (n, sym, flag, mask) = self.add_mutations(
                        cond.span,
                        &[Mutation::new(MutationType::REPLACE_WITH_FALSE, "replacing while condition with false")],
                    );
                let cond = self.fold_expr(cond);
                let block = fold::noop_fold_block(block, self);
                let mut_cond = quote_expr!(self.cx(), {
                    ::mutagen::w($cond, $n, &$sym[$flag], $mask)
                });
                P(Expr {
                    id,
                    node: ExprKind::While(mut_cond, block, opt_label),
                    span,
                    attrs,
                })
            }
            Expr {
                id,
                node: ExprKind::ForLoop(pat, expr, block, ident),
                span,
                attrs,
            } => {
                let (n, sym, flag, mask) = self.add_mutations(
                    expr.span,
                    &[
                        Mutation::new(MutationType::ITERATOR_EMPTY, "empty iterator"),
                        Mutation::new(MutationType::ITERATOR_SKIP_FIRST, "skip first element"),
                        Mutation::new(MutationType::ITERATOR_SKIP_LAST, "skip last element"),
                        Mutation::new(MutationType::ITERATOR_SKIP_BOUNDS, "skip first and last element"),
                    ],
                );
                let pat = self.fold_pat(pat);
                let expr = self.fold_expr(expr);
                let block = fold::noop_fold_block(block, self);

                let expr = quote_expr!(self.cx(), {
                    ::mutagen::forloop($expr, $n, &$sym[$flag], $mask)
                });

                P(Expr {
                    id,
                    node: ExprKind::ForLoop(pat, expr, block, ident),
                    span,
                    attrs,
                })
            }
            Expr {
                id,
                node: ExprKind::Unary(UnOp::Neg, exp),
                span,
                attrs,
            } => {
                let exp = exp.and_then(|e| {
                    let maybe_exp = match e.node {
                        ExprKind::Lit(ref lit) => {
                            self.mutate_numeric_constant_expression(&lit, true)
                        }
                        _ => None,
                    };

                    maybe_exp.unwrap_or_else(|| P(e))
                });

                P(Expr {
                    id,
                    node: ExprKind::Unary(UnOp::Neg, exp),
                    span,
                    attrs,
                })
            }
            Expr {
                id,
                node: ExprKind::Lit(lit),
                span,
                attrs,
            } => {
                lit.and_then(|l| {
                    self.mutate_numeric_constant_expression(&l, false)
                        .unwrap_or_else(|| {
                            P(Expr {
                                id,
                                node: ExprKind::Lit(P(l)),
                                span,
                                attrs,
                            })
                        })
                })
            }
            Expr {
                id,
                node: ExprKind::Path(qself, path),
                span,
                attrs,
            } => {
                if path == "self" && qself.is_none() {
                    if let Some(sym) = self.get_self_sym() {
                        let alt_self = Ident::with_empty_ctxt(sym);
                        P(Expr {
                            id,
                            node: ExprKind::Path(None, quote_path!(self.cx(), $alt_self)),
                            span,
                            attrs,
                        })
                    } else {
                        P(Expr {
                            id,
                            node: ExprKind::Path(qself, path),
                            span,
                            attrs,
                        })
                    }
                } else {
                    P(Expr {
                        id,
                        node: ExprKind::Path(qself, path),
                        span,
                        attrs,
                    })
                }
            }
            e => P(fold::noop_fold_expr(e, self)),
        }); //TODO: more expr mutations

        self.restrictions.pop();

        e
    }


    fn fold_pat(&mut self, pat: P<Pat>) -> P<Pat> {
        pat.and_then(|pattern|
            match pattern {
                Pat {
                    id,
                    node: PatKind::Range(e1, e2, e3),
                    span,
                } => {
                    // Avoid recursion on range patterns, it only literals are allowed, and mutations
                    // would potentially convert them into expressions
                    P(Pat {id, node: PatKind::Range(e1, e2, e3), span})
                },
                p => fold::noop_fold_pat(P(p), self),
            }
        )
    }

    fn fold_mac(&mut self, mac: Mac) -> Mac {
        mac
    }
}

fn int_constant_can_subtract_one(i: i128, ty: LitIntType) -> bool {
    let min: i128 = match ty {
        LitIntType::Unsuffixed | LitIntType::Unsigned(_) => 0,
        LitIntType::Signed(IntTy::Isize) => i128::from(std::i32::MIN),
        LitIntType::Signed(IntTy::I8) => i128::from(std::i8::MIN),
        LitIntType::Signed(IntTy::I16) => i128::from(std::i16::MIN),
        LitIntType::Signed(IntTy::I32) => i128::from(std::i32::MIN),
        LitIntType::Signed(IntTy::I64) => i128::from(std::i64::MIN),
        LitIntType::Signed(IntTy::I128) => std::i128::MIN,
    };

    i as i128 > min
}

static MAX_VALUES: &[u128] = &[
    std::u8::MAX as u128,
    std::u16::MAX as u128,
    std::u32::MAX as u128,
    std::u64::MAX as u128,
    std::u128::MAX as u128,
];

fn int_constant_can_add_one(i: u128, ty: LitIntType) -> bool {
    let max: u128 = match ty {
        LitIntType::Unsuffixed => {
            if MAX_VALUES.contains(&i) {
                return false;
            }
            return true;
        }
        LitIntType::Unsigned(UintTy::Usize) => u128::from(std::u32::MAX),
        LitIntType::Unsigned(UintTy::U8) => u128::from(std::u8::MAX),
        LitIntType::Unsigned(UintTy::U16) => u128::from(std::u16::MAX),
        LitIntType::Unsigned(UintTy::U32) => u128::from(std::u32::MAX),
        LitIntType::Unsigned(UintTy::U64) => u128::from(std::u64::MAX),
        LitIntType::Unsigned(UintTy::U128) => std::u128::MAX,
        LitIntType::Signed(IntTy::Isize) => std::i32::MAX as u128,
        LitIntType::Signed(IntTy::I8) => std::i8::MAX as u128,
        LitIntType::Signed(IntTy::I16) => std::i16::MAX as u128,
        LitIntType::Signed(IntTy::I32) => std::i32::MAX as u128,
        LitIntType::Signed(IntTy::I64) => std::i64::MAX as u128,
        LitIntType::Signed(IntTy::I128) => std::i128::MAX as u128,
    };

    i < max
}

// given a mutable coverage count, increment and return (index, mask)
fn coverage(coverage_count: &mut usize) -> (usize, usize) {
    let usize_bits = usize::max_value().count_ones() as usize;
    let usize_shift = usize_bits.trailing_zeros() as usize;
    let usize_mask = usize_bits - 1;
    let c = *coverage_count;
    *coverage_count += 1;
    (c >> usize_shift, 1 << (c & usize_mask))
}

fn fold_first_block(block: P<Block>, p: &mut MutatorPlugin) -> P<Block> {
    if block.rules != BlockCheckMode::Default {
        p.cx().parse_sess.span_diagnostic.fatal("mutagen: unsafe code found").raise();
    }
    let avoid = p.parent_restrictions();

    let mut pre_stmts = vec![];
    {
        let MutatorPlugin { ref mut info, ref mut m, .. } = *p;
        if let Some(&mut MethodInfo {
            ref have_output_type,
            ref interchangeables,
            ref ref_muts,
            ref coverage_sym,
            ref mut coverage_count,
            ref self_sym,
        }) = info.method_infos.last_mut()
        {
            let coverage_ident = Ident::with_empty_ctxt(*coverage_sym);
            pre_stmts.push(quote_stmt!(m.cx,
            static $coverage_ident : [::std::sync::atomic::AtomicUsize; 0] =
                [::std::sync::atomic::ATOMIC_USIZE_INIT; 0];).unwrap());
            let n = m.add_mutations(
                block.span,
                avoid,
                &[
                    Mutation::new(MutationType::RETURN_DEFAULT, "insert opportunistic return default()"),
                ],
            );
            let (flag, mask) = coverage(coverage_count);
            pre_stmts.push(
                quote_stmt!(m.cx,
                            if let Some(r) = ::mutagen::Defaulter::get_default($n, &$coverage_ident[$flag], $mask) {
                                return r;
                            }).unwrap());
            for name in have_output_type {
                let ident = Ident::with_empty_ctxt(*name);
                let n = m.add_mutations(
                    block.span,
                    avoid,
                    &[
                        Mutation::new(MutationType::RETURN_ARGUMENT, &format!("insert return {}", name)),
                    ],
                );
                let (flag, mask) = coverage(coverage_count);
                pre_stmts.push(
                    quote_stmt!(m.cx,
                        if ::mutagen::now($n, &$coverage_ident[$flag], $mask) { return $ident; }).unwrap());
            }
            for (key, ref values) in interchangeables {
                for value in values.iter() {
                    let key_ident = Ident::with_empty_ctxt(*key);
                    let value_ident = Ident::with_empty_ctxt(*value);
                    let target_key_ident = if key.as_str() == "self" { Ident::with_empty_ctxt(self_sym.unwrap()) } else { Ident::with_empty_ctxt(*key) };
                    let target_value_ident = if value.as_str() == "self" { Ident::with_empty_ctxt(self_sym.unwrap()) } else { Ident::with_empty_ctxt(*value) };
                    let n = m.add_mutations(
                        block.span,
                        avoid,
                        &[
                            Mutation::new(MutationType::EXCHANGE_ARGUMENT, &format!("exchange {} with {}", key.as_str(), value_ident)),
                        ],
                    );
                    let (flag, mask) = coverage(coverage_count);
                    pre_stmts.push(
                        quote_stmt!(m.cx,
                            let ($target_key_ident, $target_value_ident) = if ::mutagen::now($n, &$coverage_ident[$flag], $mask) {
                                ($value_ident, $key_ident)
                            } else {
                                ($key_ident, $value_ident)
                            };).unwrap(),
                    );
                }
            }
            for name in ref_muts {
                let ident = Ident::with_empty_ctxt(*name);
                let target_ident = if name.as_str() == "self" {
                    if let Some(sym) = self_sym { Ident::with_empty_ctxt(*sym) } else { ident }
                } else {
                    ident
                };
                let ident_clone = Ident::with_empty_ctxt(Symbol::gensym(&format!("_{}_clone", ident)));
                let n = m.add_mutations(
                    block.span,
                    avoid,
                    &[
                        Mutation::new(MutationType::CLONE_MUTABLE, &format!("clone mutable reference {}", ident)),
                    ],
                );
                let (flag, mask) = coverage(coverage_count);
                pre_stmts.push(quote_stmt!(m.cx, let mut $ident_clone;).unwrap());
                pre_stmts.push(
                    quote_stmt!(m.cx,
                                let $target_ident = if ::mutagen::MayClone::may_clone($ident) {
                                    $ident_clone = ::mutagen::MayClone::clone($ident,
                                        $n, &$coverage_ident[$flag], $mask);
                                    &mut $ident_clone
                                } else { $ident };).unwrap());
            }
        }
    }
    block.map(
        |Block {
             stmts,
             id,
             rules,
             span,
             recovered,
         }| {
            let mut newstmts: Vec<Stmt> = Vec::with_capacity(pre_stmts.len() + stmts.len());
            newstmts.extend(pre_stmts);
            newstmts.extend(stmts.into_iter().flat_map(|s| fold::noop_fold_stmt(s, p)));
            let coverage = mem::replace(&mut newstmts[0], quote_stmt!(p.cx(), ();).unwrap());
            let coverage_count = p.info.method_infos.last().unwrap().coverage_count;
            if coverage_count > 0 {
                let bits = usize::max_value().count_ones() as usize;
                let coverage_size = (coverage_count + bits - 1) / bits;
                let mut resizer = Resizer(coverage_size);
                let _ = mem::replace(&mut newstmts[0], resizer.fold_stmt(coverage).expect_one("?"));
            }
            Block {
                stmts: newstmts,
                id,
                rules,
                span,
                recovered,
            }
        })
}

/// combine the given `symbols` and add them to the interchangeables map
fn combine<S: Hash + Eq + Copy>(interchangeables: &mut HashMap<S, Vec<S>>, symbols: &[S]) {
    let symbol_amount = symbols.len();

    for (i, index) in symbols.iter().enumerate() {
        let change_with = (i + 1..symbol_amount).map(|i| symbols[i]).collect();
        interchangeables.insert(*index, change_with);
    }
}

/// additional position information  (which field in the given struct/enum)
#[derive(Copy, Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Debug)]
enum TyOcc {
    /// this is a subfield of a type, e.g. `Foo { x } : Foo` → `Field(x)`
    Field(Symbol),
    /// this is an index within a tuple or tuple type, e.g. `Foo(_, y): Foo` → `Index(1)`
    Index(usize),
    /// a &_ or &mut _
    Deref,
}

fn destructure_with<'t>(
    pat: &Pat,
    ty: &'t Ty,
    occ: &mut Vec<TyOcc>,
    pos: usize,
    result: &mut Vec<(Symbol, ArgTy<'t>)>,
    w: TyOcc,
) {
    occ.push(w);
    destructure_bindings(pat, ty, occ, pos, result);
    occ.pop();
}

/// Walk a pattern, call a function on each named instance
fn destructure_bindings<'t>(
    pat: &Pat,
    ty: &'t Ty,
    occ: &mut Vec<TyOcc>,
    pos: usize,
    result: &mut Vec<(Symbol, ArgTy<'t>)>,
) {
    match pat.node {
        PatKind::Ident(mode, ident, ref opt_pat) => {
            result.push((ident.name, ArgTy(mode, ty, pos, occ.clone())));
            if let Some(ref pat) = *opt_pat {
                destructure_bindings(pat, ty, occ, pos, result);
            }
        }
        PatKind::Ref(ref ref_pat, pat_mut) => {
            if let TyKind::Rptr(
                _,
                MutTy {
                    ty: ref ref_ty,
                    mutbl,
                },
            ) = ty.node
            {
                if pat_mut == mutbl && occ.is_empty() {
                    destructure_bindings(ref_pat, ref_ty, occ, pos, result);
                    return;
                }
            }
            destructure_with(ref_pat, ty, occ, pos, result, TyOcc::Deref);
        }
        PatKind::Slice(ref pats, None, _) => {
            if occ.is_empty() && pats.len() == 1 {
                if let TyKind::Slice(ref slice_ty) = ty.node {
                    destructure_bindings(&pats[0], slice_ty, occ, pos, result);
                }
            }
        }
        PatKind::Struct(_, ref fpats, _) => for fp in fpats {
            destructure_with(
                &fp.node.pat,
                ty,
                occ,
                pos,
                result,
                TyOcc::Field(fp.node.ident.name),
            );
        },
        PatKind::TupleStruct(_, ref pats, _opt_size) => for (i, p) in pats.iter().enumerate() {
            destructure_with(p, ty, occ, pos, result, TyOcc::Index(i));
        },
        PatKind::Tuple(ref pats, opt_usize) => {
            if let (true, &TyKind::Tup(ref tup)) = (occ.is_empty(), &ty.node) {
                let mut new_occs = vec![];
                for i in 0..opt_usize.unwrap_or_else(|| pats.len()) {
                    destructure_bindings(&pats[i], &tup[i], &mut new_occs, pos, result);
                }
            } else {
                for (i, p) in pats.iter().enumerate() {
                    destructure_with(p, ty, occ, pos, result, TyOcc::Index(i));
                }
            }
        }
        PatKind::Box(ref boxed_pat) => {
            if let Some(unbox_ty) = unbox(ty) {
                destructure_bindings(boxed_pat, unbox_ty, occ, pos, result);
            } else {
                destructure_with(boxed_pat, ty, occ, pos, result, TyOcc::Deref);
            }
        }
        _ => {} // wildcards, etc.
    }
}

fn unbox(ty: &Ty) -> Option<&Ty> {
    if let TyKind::Path(_, ref path) = ty.node {
        if let Some(box_seg) = path.segments.iter().last() {
            if box_seg.ident.name != "Box" {
                return None;
            }
            if let Some(ref params) = box_seg.args {
                if let GenericArgs::AngleBracketed(ref data) = **params {
                    if let GenericArg::Type(ref arg_ty) = data.args[0] {
                        return Some(arg_ty);
                    }
                }
            }
        }
    }
    None
}

fn ty_hash<H: Hasher>(ty: &Ty, pos: usize, h: &mut H) {
    match ty.node {
        TyKind::Paren(ref ty) => ty_hash(ty, pos, h),
        TyKind::Slice(ref slice) => { h.write_u8(0); ty_hash(slice, pos, h) },
        TyKind::Array(ref ty, ref lit) => { h.write_u8(1); ty_hash(ty, pos, h); get_lit(&lit.value).hash(h) },
        TyKind::Ptr(ref mutty) => { h.write_u8(2); mut_ty_hash(mutty, pos, h) },
        TyKind::Rptr(ref lt, ref mutty) => {
            h.write_u8(3);
            if let Some(ref lt) = *lt {
                lifetime_hash(lt, h);
            } else {
                h.write_usize(pos);
            }
            mut_ty_hash(mutty, pos, h)
        }
        TyKind::Never => h.write_u8(3),
        TyKind::ImplicitSelf => h.write_u8(4),
        TyKind::Tup(ref tys) => {
            h.write_u8(5);
            for ty in tys {
                ty_hash(ty, pos, h);
            }
        }
        TyKind::Path(ref qself, ref path) => {
            if path == &"Self" {
                h.write_u8(4); // same as ImplicitSelf
            } else {
                h.write_u8(6);
                if let Some(ref qself) = *qself {
                    h.write_usize(qself.position);
                    ty_hash(&qself.ty, pos, h);
                }
                path_hash(path, pos, h);
            }
        }
        TyKind::TraitObject(ref bounds, ref syn) => {
            h.write_u8(7);
            for bound in bounds {
                generic_bound_hash(bound, pos, h);
            }
            syn.hash(h);
        }
        TyKind::ImplTrait(_, ref bounds) => {
            h.write_u8(8);
            for bound in bounds {
                generic_bound_hash(bound, pos, h);
            }
        }
        // don't care about the other values
        _ => ty.hash(h)
    }
}

fn ty_equal(a: &Ty, b: &Ty, inout: bool) -> bool {
    match (&a.node, &b.node) {
        (&TyKind::Paren(ref aty), _) => ty_equal(&aty, b, inout),
        (_, &TyKind::Paren(ref bty)) => ty_equal(a, &bty, inout),
        (&TyKind::Slice(ref aslice), &TyKind::Slice(ref bslice)) => ty_equal(aslice, bslice, inout),
        (&TyKind::Array(ref aty, ref alit), &TyKind::Array(ref bty, ref blit)) => {
            ty_equal(&aty, &bty, inout) && get_lit(&alit.value).map_or(false, |a| Some(a) == get_lit(&blit.value))
        }
        (&TyKind::Ptr(ref amut), &TyKind::Ptr(ref bmut)) => ty_mut_equal(amut, bmut, inout),
        (&TyKind::Rptr(ref alt, ref amut), &TyKind::Rptr(ref blt, ref bmut)) => {
            if let (&Some(ref alt), &Some(ref blt)) = (alt, blt) {
                lifetime_equal(alt, blt) && ty_mut_equal(amut, bmut, inout)
            } else {
                inout && alt.is_none() && blt.is_none() && ty_mut_equal(amut, bmut, inout)
            }
        }
        (&TyKind::Never, &TyKind::Never) |
        (&TyKind::ImplicitSelf, &TyKind::ImplicitSelf) => true,
        (&TyKind::Tup(ref atys), &TyKind::Tup(ref btys)) => {
            vecd(atys, btys, |a, b| ty_equal(a, b, inout))
        }
        (&TyKind::Path(ref aq, ref apath), &TyKind::Path(ref bq, ref bpath)) => {
            optd(&aq, &bq, |a, b|
                ty_equal(&a.ty, &b.ty, inout) && a.position == b.position) && path_equal(apath, bpath, inout)
        }
        (&TyKind::Path(None, ref path), &TyKind::ImplicitSelf) |
        (&TyKind::ImplicitSelf, &TyKind::Path(None, ref path)) => {
            match_path(path, &["Self"])
        }
        (&TyKind::TraitObject(ref abounds, ref asyn), &TyKind::TraitObject(ref bbounds, ref bsyn)) => {
            asyn == bsyn && vecd(abounds, bbounds, |a, b| generic_bound_equal(a, b, inout))
        }
        (&TyKind::ImplTrait(_, ref abounds), &TyKind::ImplTrait(_, ref bbounds)) => {
            vecd(abounds, bbounds, | a, b| generic_bound_equal(a, b, inout))
        }
        _ => false, // we can safely ignore inferred types, type macros and error types
    }
}

fn vecd<T, F: Fn(&T, &T) -> bool>(a: &[T], b: &[T], f: F) -> bool {
    a.len() == b.len() && a.into_iter().zip(b.into_iter()).all(|(x, y)| f(&*x, &*y))
}

fn optd<T, F: Fn(&T, &T) -> bool>(a: &Option<T>, b: &Option<T>, f: F) -> bool {
    a.as_ref().map_or_else(|| b.is_none(), |aref| b.as_ref().map_or(false, |bref| f(aref, bref)))
}

fn mut_ty_hash<H: Hasher>(m: &MutTy, pos: usize, h: &mut H) {
    ty_hash(&m.ty, pos, h);
    m.mutbl.hash(h);
}

fn ty_mut_equal(a: &MutTy, b: &MutTy, inout: bool) -> bool {
    ty_equal(&a.ty, &b.ty, inout) && a.mutbl == b.mutbl
}

fn ty_bindings_equal(a: &TypeBinding, b: &TypeBinding, inout: bool) -> bool {
    a.ident == b.ident && ty_equal(&a.ty, &b.ty, inout)
}

fn path_hash<H: Hasher>(p: &Path, pos: usize, h: &mut H) {
    let pos = if is_whitelisted_path(p) { usize::max_value() } else { pos };
    for segment in &p.segments {
        path_segment_hash(segment, pos, h);
    }
}

fn path_equal(a: &Path, b: &Path, inout: bool) -> bool {
    vecd(&a.segments, &b.segments, |aseg, bseg| path_segment_equal(aseg, bseg, inout || is_whitelisted_path(a)))
}

// for now we restrict ourselves to primitive types, just to be sure
static LIFETIME_LESS_PATHS: &[&[&str]] = &[
    &["u8"], &["u16"], &["u32"], &["u64"], &["u128"], &["usize"],
    &["i8"], &["i16"], &["i32"], &["i64"], &["i128"], &["isize"],
    &["char"], &["bool"], &["Self"]]; // Self

fn is_whitelisted_path(path: &Path) -> bool {
    LIFETIME_LESS_PATHS.iter().any(|segs| match_path(path, segs))
}

fn generic_arg_hash<H: Hasher>(arg: &GenericArg, pos: usize, h: &mut H) {
    match *arg {
        GenericArg::Lifetime(ref lt) => lifetime_hash(lt, h),
        GenericArg::Type(ref ty) => ty_hash(ty, pos, h),
    }
}

fn generic_arg_equal(a: &GenericArg, b: &GenericArg, inout: bool) -> bool {
    match (a, b) {
        (&GenericArg::Lifetime(ref alt), &GenericArg::Lifetime(ref blt)) =>
            lifetime_equal(alt, blt),
        (&GenericArg::Type(ref aty), &GenericArg::Type(ref bty)) => ty_equal(aty, bty, inout),
        _ => false
    }
}

fn path_segment_hash<H: Hasher>(seg: &PathSegment, pos: usize, h: &mut H) {
    seg.ident.hash(h);
    if let Some(ref params) = seg.args {
        match **params {
            GenericArgs::AngleBracketed(ref data) => {
                h.write_u8(0);
                for arg in &data.args {
                    generic_arg_hash(arg, pos, h);
                }
                for binding in &data.bindings {
                    //TODO: ident_hash(binding.ident)
                    ty_hash(&binding.ty, pos, h);
                }
            }
            GenericArgs::Parenthesized(ref data) => {
                for i in &data.inputs {
                    ty_hash(i, pos, h);
                }
                if let Some(ref ty) = data.output {
                    ty_hash(ty, pos, h);
                }
            }
        }
    }
}

fn path_segment_equal(a: &PathSegment, b: &PathSegment, inout: bool) -> bool {
    a.ident == b.ident && optd(&a.args, &b.args, |a, b| match (&**a, &**b) {
        (&GenericArgs::AngleBracketed(ref adata), &GenericArgs::AngleBracketed(ref bdata)) => {
            vecd(&adata.args, &bdata.args, |a, b| generic_arg_equal(a, b, inout)) &&
                vecd(&adata.bindings, &bdata.bindings, |a, b| ty_bindings_equal(a, b, inout))
        }
        (&GenericArgs::Parenthesized(ref adata), &GenericArgs::Parenthesized(ref bdata)) => {
            vecd(&adata.inputs, &bdata.inputs, |a, b| ty_equal(a, b, inout)) &&
                optd(&adata.output, &bdata.output, |a, b| ty_equal(a, b, inout))
        }
        _ => false
    })
}

fn lifetime_hash<H: Hasher>(l: &Lifetime, h: &mut H) {
    l.ident.name.hash(h)
}

fn lifetime_equal(a: &Lifetime, b: &Lifetime) -> bool {
    a.ident == b.ident
}

fn generic_param_hash<H: Hasher>(p: &GenericParam, pos: usize, h: &mut H) {
    p.ident.name.hash(h);
    for bound in &p.bounds {
        generic_bound_hash(bound, pos, h);
    }
    match p.kind {
        GenericParamKind::Lifetime => h.write_u8(37),
        GenericParamKind::Type { default: ref typaram } => {
            h.write_u8(43);
            if let Some(t) = typaram {
                ty_hash(t, pos, h);
            }
        }
    }
}

fn generic_param_equal(a: &GenericParam, b: &GenericParam, inout: bool) -> bool {
    a.ident == b.ident && vecd(&a.bounds, &b.bounds, |a, b| generic_bound_equal(a, b, inout)) &&
    match (&a.kind, &b.kind) {
        (&GenericParamKind::Lifetime, &GenericParamKind::Lifetime) |
        (&GenericParamKind::Type { default: None },
         &GenericParamKind::Type { default: None }) => true,
        (&GenericParamKind::Type { default: Some(ref aty) },
         &GenericParamKind::Type { default: Some(ref bty) }) => ty_equal(aty, bty, inout),
        _ => false
    }
}

fn trait_ref_hash<H: Hasher>(t: &PolyTraitRef, pos: usize, h: &mut H) {
    for gp in &t.bound_generic_params {
        generic_param_hash(gp, pos, h);
    }
    path_hash(&t.trait_ref.path, pos, h);
}

fn trait_ref_equal(a: &PolyTraitRef, b: &PolyTraitRef, inout: bool) -> bool {
    vecd(&a.bound_generic_params, &b.bound_generic_params, |a, b| generic_param_equal(a, b, inout)) &&
        path_equal(&a.trait_ref.path, &b.trait_ref.path, inout)
}

fn generic_bound_hash<H: Hasher>(tpb: &GenericBound, pos: usize, h: &mut H) {
    match *tpb {
        GenericBound::Trait(ref t, ref m) => {
            h.write_u8(0);
            trait_ref_hash(t, pos, h);
            m.hash(h);
        },
        GenericBound::Outlives(ref lifetime) => {
            h.write_u8(1);
            lifetime_hash(lifetime, h);
        }
    }
}

fn generic_bound_equal(a: &GenericBound, b: &GenericBound, inout: bool) -> bool {
    match (a, b) {
        (&GenericBound::Trait(ref at, ref amod), &GenericBound::Trait(ref bt, ref bmod)) => {
            amod == bmod && trait_ref_equal(at, bt, inout)
        }
        (&GenericBound::Outlives(ref alt), &GenericBound::Outlives(ref blt)) => {
            lifetime_equal(alt, blt)
        }
        _ => false
    }
}

fn is_ty_ref_mut(ty: &Ty) -> bool {
    if let TyKind::Rptr(_, MutTy { mutbl: Mutability::Mutable, .. }) = ty.node {
        true
    } else {
        false
    }
}

fn match_path(path: &Path, pat: &[&str]) -> bool {
    path.segments
        .iter()
        .rev()
        .zip(pat.iter().rev())
        .all(|(a, b)| &a.ident.name == b)
}

fn get_lit(expr: &Expr) -> Option<u128> {
    if let ExprKind::Lit(ref lit) = expr.node {
        if let LitKind::Int(val, _) = lit.node {
            return Some(val);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use syntax::ast::{IntTy, LitIntType};

    #[test]
    fn test_can_add_one() {
        let examples = [
            (std::u8::MAX as u128, LitIntType::Unsuffixed, false),
            ((std::u8::MAX as u128) + 1, LitIntType::Unsuffixed, true),
            ((std::u8::MAX as u128) - 1, LitIntType::Unsuffixed, true),
            (
                (std::i128::MAX as u128),
                LitIntType::Signed(IntTy::I128),
                false,
            ),
            ((std::u128::MAX as u128) - 1, LitIntType::Unsigned(UintTy::U128), true),
        ];

        examples.iter().for_each(|test| {
            let actual = int_constant_can_add_one(test.0, test.1);

            assert_eq!(actual, test.2);
        });
    }

    #[test]
    fn test_can_subtract_one() {
        let examples = [
            (1 as i128, LitIntType::Unsuffixed, true),
            (0 as i128, LitIntType::Unsuffixed, false),
            (std::i8::MIN as i128, LitIntType::Signed(IntTy::I8), false),
            (std::i8::MIN as i128 + 1, LitIntType::Signed(IntTy::I8), true),
            (std::i128::MIN as i128, LitIntType::Signed(IntTy::I128), false),
            (
                std::i128::MIN as i128 + 1,
                LitIntType::Signed(IntTy::I128),
                true,
            ),
        ];

        examples.iter().for_each(|test| {
            let actual = int_constant_can_subtract_one(test.0, test.1);

            assert_eq!(actual, test.2);
        });
    }

    #[test]
    fn test_combine() {
        let a = "a";
        let b = "b";
        let c = "c";
        let d = "d";

        let symbols = [a, b, c, d];

        let mut interchangeables = HashMap::new();
        combine(&mut interchangeables, &symbols);

        assert_eq!(interchangeables[&a], &[b, c, d]);
        assert_eq!(interchangeables[&b], &[c, d]);
        assert_eq!(interchangeables[&c], &[d]);
    }
}
