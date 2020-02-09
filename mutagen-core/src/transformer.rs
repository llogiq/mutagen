use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::fold::Fold;

mod arg_ast;
mod mutate_args;
pub mod transform_context;
pub mod transform_info;
pub use transform_context::TransformContext;

use crate::mutator::*;
use transform_info::SharedTransformInfo;

pub fn do_transform_item(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = match syn::parse2::<syn::Item>(input) {
        Ok(ast) => ast,
        Err(e) => return TokenStream::from(e.to_compile_error()),
    };
    MutagenTransformerBundle::setup_from_attr(args.into()).mutagen_process_item(input)
}

pub enum MutagenTransformer {
    Expr(Box<MutagenExprTransformer>),
    Stmt(Box<MutagenStmtTransformer>),
}

pub struct MutagenTransformerBundle {
    transform_info: SharedTransformInfo,
    transform_context: TransformContext,
    expr_transformers: Vec<Box<MutagenExprTransformer>>,
    stmt_transformers: Vec<Box<MutagenStmtTransformer>>,
}

/// function-type that describes expression-transformers.
///
// the transformer should not inspect the expression recursively since recursion is performed by the `MutagenTransformerBundle`
type MutagenExprTransformer =
    dyn FnMut(syn::Expr, &SharedTransformInfo, &TransformContext) -> syn::Expr;

/// function-type that describes expression-transformers.
///
// the transformer should not inspect the expression recursively since recursion is performed by the `MutagenTransformerBundle`
type MutagenStmtTransformer =
    dyn FnMut(syn::Stmt, &SharedTransformInfo, &TransformContext) -> syn::Stmt;

impl Fold for MutagenTransformerBundle {
    fn fold_expr(&mut self, e: syn::Expr) -> syn::Expr {
        // save the original expr into the context
        let old_expr = self.transform_context.original_expr.replace(e.clone());

        // transform content of the expression first
        let mut result = syn::fold::fold_expr(self, e);

        // call all transformers on this expression
        for transformer in &mut self.expr_transformers {
            result = transformer(result, &self.transform_info, &self.transform_context);
        }

        // reset original_stmt to original state
        self.transform_context.original_expr = old_expr;
        result
    }

    fn fold_stmt(&mut self, s: syn::Stmt) -> syn::Stmt {
        // save the original stmt into the context
        let old_stmt = self.transform_context.original_stmt.replace(s.clone());

        // transform content of the statement first
        let mut result = syn::fold::fold_stmt(self, s);

        // call all transformers on this statement
        for transformer in &mut self.stmt_transformers {
            result = transformer(result, &self.transform_info, &self.transform_context);
        }

        // reset original_stmt to original state
        self.transform_context.original_stmt = old_stmt;
        result
    }

    fn fold_item_fn(&mut self, i: syn::ItemFn) -> syn::ItemFn {
        // do not mutate const functions
        if i.sig.constness.is_some() {
            return i;
        }
        // do not mutate unsafe functions
        if i.sig.unsafety.is_some() {
            return i;
        }

        // insert the new functionname into context
        let old_fn_name = self
            .transform_context
            .fn_name
            .replace(i.sig.ident.to_string());

        // do transformations
        let result = syn::fold::fold_item_fn(self, i);

        // restore old context
        self.transform_context.fn_name = old_fn_name;

        result
    }

    fn fold_impl_item_method(&mut self, i: syn::ImplItemMethod) -> syn::ImplItemMethod {
        // do not mutate const functions
        if i.sig.constness.is_some() {
            return i;
        }
        // do not mutate unsafe functions
        if i.sig.unsafety.is_some() {
            return i;
        }

        // insert the new functionname into context
        let old_fn_name = self
            .transform_context
            .fn_name
            .replace(i.sig.ident.to_string());

        // do transformations
        let result = syn::fold::fold_impl_item_method(self, i);

        // restore old context
        self.transform_context.fn_name = old_fn_name;

        result
    }

    fn fold_item_impl(&mut self, i: syn::ItemImpl) -> syn::ItemImpl {
        // insert the new item name into context
        let old_fn_name = self
            .transform_context
            .impl_name
            .replace(i.self_ty.to_token_stream().to_string());

        // do transformations
        let result = syn::fold::fold_item_impl(self, i);

        // restore old context
        self.transform_context.impl_name = old_fn_name;

        result
    }

    fn fold_expr_repeat(&mut self, e: syn::ExprRepeat) -> syn::ExprRepeat {
        let syn::ExprRepeat {
            attrs,
            bracket_token,
            expr,
            semi_token,
            len,
        } = e;

        // mutate expr only, `len` is constant and should not be mutated
        let expr = Box::new(syn::fold::fold_expr(self, *expr));

        syn::ExprRepeat {
            attrs,
            bracket_token,
            expr,
            semi_token,
            len,
        }
    }

    fn fold_foreign_item_fn(&mut self, i: syn::ForeignItemFn) -> syn::ForeignItemFn {
        // do not mutate const functions
        if i.sig.constness.is_some() {
            return i;
        }
        if i.sig.unsafety.is_some() {
            return i;
        }

        // insert the new functionname into context
        let old_fn_name = self
            .transform_context
            .fn_name
            .replace(i.sig.ident.to_string());

        // do transformations
        let result = syn::fold::fold_foreign_item_fn(self, i);

        // restore old context
        self.transform_context.fn_name = old_fn_name;

        result
    }

    fn fold_pat(&mut self, i: syn::Pat) -> syn::Pat {
        // do not mutate patterns
        i
    }

    fn fold_item_const(&mut self, i: syn::ItemConst) -> syn::ItemConst {
        // do not mutate const-items
        i
    }

    fn fold_item_static(&mut self, i: syn::ItemStatic) -> syn::ItemStatic {
        // do not mutate static items
        i
    }

    fn fold_type(&mut self, t: syn::Type) -> syn::Type {
        // do not mutate types
        t
    }

    fn fold_expr_unsafe(&mut self, e: syn::ExprUnsafe) -> syn::ExprUnsafe {
        // do not mutate unsafe blocks
        e
    }
}

impl MutagenTransformerBundle {
    pub fn mutagen_process_item(&mut self, target: syn::Item) -> TokenStream {
        let stream = self.fold_item(target).into_token_stream();
        self.transform_info.check_mutations();
        stream
    }

    pub fn mk_transformer(
        transformer_name: &str,
        _transformer_args: &[String],
    ) -> MutagenTransformer {
        match transformer_name {
            "lit_int" => MutagenTransformer::Expr(Box::new(mutator_lit_int::transform)),
            "lit_bool" => MutagenTransformer::Expr(Box::new(mutator_lit_bool::transform)),
            "unop_not" => MutagenTransformer::Expr(Box::new(mutator_unop_not::transform)),
            "binop_bit" => MutagenTransformer::Expr(Box::new(mutator_binop_bit::transform)),
            "binop_num" => MutagenTransformer::Expr(Box::new(mutator_binop_num::transform)),
            "binop_eq" => MutagenTransformer::Expr(Box::new(mutator_binop_eq::transform)),
            "binop_cmp" => MutagenTransformer::Expr(Box::new(mutator_binop_cmp::transform)),
            "binop_bool" => MutagenTransformer::Expr(Box::new(mutator_binop_bool::transform)),
            "stmt_call" => MutagenTransformer::Stmt(Box::new(mutator_stmt_call::transform)),
            _ => panic!("unknown transformer {}", transformer_name),
        }
    }

    // this funciton gives a vec of all transformers, in order they are executed
    pub fn all_transformers() -> Vec<String> {
        [
            "lit_int",
            "lit_bool",
            "unop_not",
            "binop_bit",
            "binop_num",
            "binop_eq",
            "binop_cmp",
            "binop_bool",
            "stmt_call",
        ]
        .iter()
        .copied()
        .map(ToOwned::to_owned)
        .collect()
    }

    /// parse the arguments of the `#[mutate]` attribute
    fn setup_from_attr(args: TokenStream) -> Self {
        use self::mutate_args::*;

        let options = ArgOptions::parse(args).expect("invalid options");

        // create transform_info
        let transform_info: SharedTransformInfo = match options.conf {
            Conf::Global => SharedTransformInfo::global_info(),
            Conf::Local(local_conf) => SharedTransformInfo::local_info(local_conf),
        };

        // create transformers
        let transformers = match options.transformers {
            Transformers::All => Self::all_transformers(),
            Transformers::Only(list) => {
                let mut transformers = list.transformers;
                transformers.sort_by_key(|t| TRANSFORMER_ORDER[t]);
                transformers
            }
            Transformers::Not(list) => {
                let mut transformers = Self::all_transformers();
                for l in &list.transformers {
                    // transformers.remove_item(l)
                    if let Some(pos) = transformers.iter().position(|x| *x == *l) {
                        transformers.remove(pos);
                    }
                }
                transformers
            }
        };
        let mut expr_transformers = Vec::new();
        let mut stmt_transformers = Vec::new();
        for t in &transformers {
            let t = Self::mk_transformer(t, &[]);
            match t {
                MutagenTransformer::Expr(t) => expr_transformers.push(t),
                MutagenTransformer::Stmt(t) => stmt_transformers.push(t),
            }
        }

        let transform_context = TransformContext::default();

        Self {
            transform_context,
            transform_info,
            expr_transformers,
            stmt_transformers,
        }
    }
}

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref TRANSFORMER_ORDER: HashMap<String, usize> = {
        MutagenTransformerBundle::all_transformers()
            .into_iter()
            .enumerate()
            .map(|(i, s)| (s, i))
            .collect()
    };
}
