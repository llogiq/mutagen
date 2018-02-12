#![feature(plugin_registrar, quote, rustc_private, custom_attribute, try_from)]

extern crate rustc_plugin;
extern crate syntax;

use Mutation::*;
use rustc_plugin::registry::Registry;
use std::collections::HashMap;
use std::convert::TryFrom;
use syntax::ast::*;
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt, SyntaxExtension};
use syntax::fold::{self, Folder};
use syntax::ptr::P;
use syntax::symbol::Symbol;
use syntax::util::small_vector::SmallVector;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(Symbol::intern("mutate"),
        SyntaxExtension::MultiModifier(Box::new(mutator)));
}

pub fn mutator(cx: &mut ExtCtxt, _span: Span, _mi: &MetaItem,
                          a: Annotatable) -> Annotatable {
    let mut p = MutatorPlugin::new(cx);
    match a {
        Annotatable::Item(i) => Annotatable::Item(
            p.fold_item(i).expect_one("expected exactly one item")),
        Annotatable::TraitItem(i) => Annotatable::TraitItem(
            i.map(|i| p.fold_trait_item(i)
                       .expect_one("expected exactly one item"))),
        Annotatable::ImplItem(i) => Annotatable::ImplItem(
            i.map(|i| p.fold_impl_item(i)
                       .expect_one("expected exactly one item"))),
    }
}



struct MethodInfo {
    returns_default: bool,
    interchangeables: HashMap<Symbol, Vec<Symbol>>,
}

/// The various mutations we can do
enum Mutation {
    /// an early return (if the return value has a Default impl)
    EarlyReturn,
    /// replace a literal value with one of a selection
    ReplaceLiteral(usize),
    /// Change `x == y` to `true`, `false` or `x != y`
    AndAnd,
    OrOr,
    IfCondition,
    WhileCondition,
    Equal,
    /// Change `x != y` to `true`, `false` or `x == y`
    NotEqual,
    /// Change `x > y` to `true`, `false`, `x >= y`, `x < y`, `x <= y`,
    /// `x == y` or `x != y`
    /// (also works with `x < y` by switching operands)
    GreaterThan,
    /// Change `x >= y` to `true`, `false`, `x > y`, `x < y`, `x <= y`,
    /// `x == y` or `x != y`
    /// (also works with `x <= y` by switching operands)
    GreaterEqual,
}


impl Mutation {
    /// How many counts does this mutation add?
    fn count(&self) -> usize {
        match *self {
            ReplaceLiteral(n) => n,
            Equal | NotEqual => 3,
            GreaterThan | GreaterEqual => 7,
            _ => 1,
        }
    }
}

/// The MutatorPlugin
pub struct MutatorPlugin<'a, 'cx: 'a> {
    /// context for quoting
    cx: &'a mut ExtCtxt<'cx>,
    /// a stack of method infos
    method_infos: Vec<MethodInfo>,
    /// a sequence of mutations
    mutations: Vec<Mutation>,
    /// the current mutation count, starting from 1
    current_count: usize
}

impl<'a, 'cx> MutatorPlugin<'a, 'cx> {
    fn new(cx: &'a mut ExtCtxt<'cx>) -> Self {
        MutatorPlugin {
            cx,
            method_infos: vec![],
            mutations: vec![],
            current_count: 1
        }
    }

    /// increment the mutation count by `n`, return the previous value
    fn next(&mut self, n: usize) {
        let result = self.current_count;
        self.current_count += n;
        result
    }

    fn start_fn(&mut self, decl: &FnDecl) {
        let returns_default = match decl.output {
            FunctionRetTy::Default(_) => true,
            FunctionRetTy::Ty(ref ty) => is_ty_default(ty, None)
        };
        // add arguments of same type, so we can switch them?
        let mut argtypes : HashMap<Symbol, &Ty> = HashMap::new();
        let mut typeargs : HashMap<&Ty, Vec<Symbol>> = HashMap::new();
        for arg in &decl.inputs {
            if let Some(name) = get_pat_name(&arg.pat) {
                argtypes.insert(name, &*arg.ty);
                typeargs.entry(&arg.ty).or_insert(vec![]).push(name);
            }
        }
        let mut interchangeables : HashMap<Symbol, Vec<Symbol>> = HashMap::new();
        for (name, ty) in argtypes {
            let alt = &typeargs[ty];
            if alt.len() > 1 {
                interchangeables.insert(name, alt.iter()
                                             .filter(|n| **n != name)
                                             .cloned()
                                             .collect::<Vec<_>>());
            }
        }
        self.method_infos.push(MethodInfo { returns_default, interchangeables });
    }

    fn end_fn(&mut self) {
        let info =self.method_infos.pop();
        assert!(info.is_some());
    }
}

impl<'a, 'cx> Folder for MutatorPlugin<'a, 'cx> {
    fn fold_impl_item(&mut self, i: ImplItem) -> SmallVector<ImplItem> {
        let mut is_fn = false;
        if let ImplItemKind::Method(ref sig, _) = i.node {
            self.start_fn(&sig.decl);
            is_fn = true;
        }
        let item = fold::noop_fold_impl_item(i, self);
        if is_fn {
            self.end_fn();
        }
        item
    }

    fn fold_trait_item(&mut self, i: TraitItem) -> SmallVector<TraitItem> {
        let mut is_fn = false;
        if let TraitItemKind::Method(ref sig, Some(_)) = i.node {
            self.start_fn(&sig.decl);
            is_fn = true;
        }
        let item = fold::noop_fold_trait_item(i, self);
        if is_fn {
            self.end_fn();
        }
        item
    }

    fn fold_item_simple(&mut self, i: Item) -> Item {
        let mut is_fn = false;
        if let ItemKind::Fn(ref decl, _, _, _, _, _) = i.node {
            self.start_fn(&decl);
            is_fn = true;
        }
        let item = fold::noop_fold_item_simple(i, self);
        if is_fn {
            self.end_fn();
        }
        item
    }

    fn fold_block(&mut self, block: P<Block>) -> P<Block> {
        if self.method_infos.last().map_or(false, |i| i.returns_default) {
            block.map(|b| {
                let Block { stmts, id, rules, span, recovered } = b;
                let mut newstmts : Vec<Stmt> = Vec::with_capacity(stmts.len() + 1);
                let n = self.next(1);
                newstmts.push(quote_stmt!(self.cx,
                    if mutagen::MU.now($n) { return Default::default(); }).unwrap());
                newstmts.extend(stmts.into_iter().flat_map(|s| fold::noop_fold_stmt(s, self)));
                Block { stmts: newstmts, id, rules, span, recovered }
            })
        } else {
            fold::noop_fold_block(block, self)
        }
    }

    fn fold_expr(&mut self, expr: P<Expr>) -> P<Expr> {
        fold::noop_fold_expr(expr, self) //TODO
    }
}

fn get_pat_name(pat: &Pat) -> Option<Symbol> {
    if let PatKind::Ident(_, i, _) = pat.node {
        Some(i.node.name)
    } else {
        None
    }
}

static ALWAYS_DEFAULT : &[&[&str]] = &[
    &["u8"], &["u16"], &["u32"], &["u64"], &["u128"], &["usize"],
    &["i8"], &["i16"], &["i32"], &["i64"], &["i128"], &["isize"],
    &["vec", "Vec"],
    &["option", "Option"],
    &["char"],
    &["str"],
    &["string", "String"],
    &["BTreeMap"],
    &["BTreeSet"],
    &["HashMap"],
    &["HashSet"],
    &["vec_deque", "VecDeque"],
    &["linked_list", "LinkedList"],
    &["heap", "Heap"],
    &["BinaryHeap"],
    &["time", "Duration"],
    &["iter", "Empty"],
    &["fmt", "Error"],
    &["hash", "SipHasher"], &["hash", "SipHasher24"],
    &["hash", "BuildHasherDefault"],
    &["collections", "hash_map", "DefaultHasher"],
    &["collections", "hash_map", "RandomState"],
    &["ffi", "CStr"],
    &["ffi", "CString"],
    &["ffi", "OsStr"],
    &["ffi", "OsString"],
    &["path", "PathBuf"],
    &["sync", "CondVar"],
];

static DEFAULT_IF_ARG : &[&[&str]] = &[
    &["boxed", "Box"],
    &["rc", "Rc"],
    &["rc", "Weak"],
    &["arc", "Arc"],
    &["arc", "Weak"],
    &["cell", "Cell"],
    &["cell", "RefCell"],
    &["cell", "UnsafeCell"],
    &["num", "Wrapping"],
    &["sync", "atomic", "AtomicPtr"],
    &["sync", "atomic", "AtomicBool"],
    &["sync", "atomic", "AtomicU8"],
    &["sync", "atomic", "AtomicU16"],
    &["sync", "atomic", "AtomicU32"],
    &["sync", "atomic", "AtomicU64"],
    &["sync", "atomic", "AtomicUsize"],
    &["sync", "atomic", "AtomicI8"],
    &["sync", "atomic", "AtomicI16"],
    &["sync", "atomic", "AtomicI32"],
    &["sync", "atomic", "AtomicI64"],
    &["sync", "atomic", "AtomicIsize"],
    &["sync", "Mutex"],
    &["sync", "RwLock"],
    &["mem", "ManuallyDrop"],
    ];

fn is_ty_default(ty: &Ty, self_ty: Option<&Ty>) -> bool {
    match ty.node {
        TyKind::Slice(_) | TyKind::Never => true,
        TyKind::Rptr(_lt, MutTy { ty: ref pty, .. }) => {
            match pty.node {
                TyKind::Slice(_) => true,
                TyKind::Path(_, ref ty_path) => match_path(ty_path, &["str"]),
                _ => false
            }
        },
        TyKind::Paren(ref t) => is_ty_default(t, self_ty),
        TyKind::Array(ref inner, ref len) => is_ty_default(inner, self_ty) &&
            get_lit(len).map_or(false, |n| n <= 32),
        TyKind::Tup(ref inners) => inners.len() <= 12 &&
            inners.iter().all(|t| is_ty_default(&*t, self_ty)),
        TyKind::Path(ref _qself, ref ty_path) =>
            is_path_default(ty_path, self_ty),
        TyKind::TraitObject(ref bounds, _) | TyKind::ImplTrait(ref bounds) => {
            bounds.iter().any(|bound| {
                if let TraitTyParamBound(ref poly_trait, _) = *bound {
                    poly_trait.trait_ref.path.segments.last().map_or(false,
                        |s| s.identifier.name == "Default")
                } else {
                    false
                }
            })
        },
        TyKind::ImplicitSelf => self_ty.map_or(false, |t| is_ty_default(t, None)),
        TyKind::Typeof(ref expr) => is_expr_default(expr, self_ty),
        _ => false
    }
}

fn is_expr_default(expr: &Expr, self_ty: Option<&Ty>) -> bool {
    match expr.node {
        ExprKind::Path(_, ref path) => is_path_default(path, self_ty),
        ExprKind::Paren(ref e) => is_expr_default(e, self_ty),
        ExprKind::AddrOf(_, ref e) => {
            match e.node {
                ExprKind::Array(ref exprs) => exprs.len() == 1,
                ExprKind::Path(_, ref path) => match_path(path, &["str"]),
                _ => false
            }
        },
        ExprKind::Repeat(ref e, ref len) =>
            is_expr_default(e, self_ty)
            && get_lit(len).map_or(false, |n| n <= 32),
        ExprKind::Array(ref exprs) =>
            exprs.len() == 1, // = Slice
        ExprKind::Tup(ref exprs) =>
            exprs.len() <= 12
            && exprs.iter().all(|e| is_expr_default(e, self_ty)),
        _ => false
    }
}

fn is_path_default(ty_path: &Path, self_ty: Option<&Ty>) -> bool {
    if ALWAYS_DEFAULT.iter().any(|p| match_path(ty_path, p)) {
        return true
    }
    for path in DEFAULT_IF_ARG {
        if match_path(ty_path, path) {
            return ty_path.segments.last().map_or(false, |s| {
                s.parameters.as_ref().map_or(false, |p| {
                    if let AngleBracketed(ref data) = **p {
                        data.types.len() == 1 &&
                            is_ty_default(&*data.types[0], self_ty)
                    } else {
                        false
                    }
                })
            })
        }
    }
    // TODO: Cow
    false
}

fn match_path(path: &Path, pat: &[&str]) -> bool {
    path.segments.iter().rev().zip(
              pat.iter().rev()).all(|(a, b)| &a.identifier.name == b)
}

fn get_lit(expr: &Expr) -> Option<usize> {
    if let ExprKind::Lit(ref lit) = expr.node {
        if let LitKind::Int(val, _) = lit.node {
            return usize::try_from(val).ok();
        }
    }
    None
}
