use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::fold::Fold;
use syn::{parse2, Expr, Item, ItemFn, Stmt};

mod arg_ast;
mod mutate_args;
pub mod transform_context;
pub mod transform_info;

use crate::mutator::*;
use transform_context::TransformContext;
use transform_info::SharedTransformInfo;

pub fn do_transform_item(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = match parse2::<Item>(input) {
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
type MutagenExprTransformer = dyn FnMut(Expr, &SharedTransformInfo, &TransformContext) -> Expr;

/// function-type that describes expression-transformers.
///
// the transformer should not inspect the expression recursively since recursion is performed by the `MutagenTransformerBundle`
type MutagenStmtTransformer = dyn FnMut(Stmt, &SharedTransformInfo, &TransformContext) -> Stmt;

impl Fold for MutagenTransformerBundle {
    fn fold_expr(&mut self, e: Expr) -> Expr {
        // transform content of the expression first
        let mut result = syn::fold::fold_expr(self, e);

        // call all transformers on this expression
        for transformer in &mut self.expr_transformers {
            result = transformer(result, &self.transform_info, &self.transform_context);
        }
        result
    }

    fn fold_stmt(&mut self, s: Stmt) -> Stmt {
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

    // TODO: comment
    fn fold_item_fn(&mut self, i: ItemFn) -> ItemFn {
        let old_fn_name = self
            .transform_context
            .fn_name
            .replace(i.sig.ident.to_string());
        let result = syn::fold::fold_item_fn(self, i);
        self.transform_context.fn_name = old_fn_name;
        result
    }
}

impl MutagenTransformerBundle {
    pub fn mutagen_process_item(&mut self, target: Item) -> TokenStream {
        let stream = self.fold_item(target).into_token_stream();
        self.transform_info.check_mutations();
        stream
    }

    pub fn mk_transformer(
        transformer_name: &str,
        _transformer_args: &[String],
    ) -> MutagenTransformer {
        match transformer_name {
            "lit_int" => MutagenTransformer::Expr(Box::new(MutatorLitInt::transform)),
            "lit_bool" => MutagenTransformer::Expr(Box::new(MutatorLitBool::transform)),
            "unop_not" => MutagenTransformer::Expr(Box::new(MutatorUnopNot::transform)),
            "binop_add" => MutagenTransformer::Expr(Box::new(MutatorBinopAdd::transform)),
            "binop_eq" => MutagenTransformer::Expr(Box::new(MutatorBinopEq::transform)),
            "binop_cmp" => MutagenTransformer::Expr(Box::new(MutatorBinopCmp::transform)),
            "binop_bool" => MutagenTransformer::Expr(Box::new(MutatorBinopBool::transform)),
            "stmt_call" => MutagenTransformer::Stmt(Box::new(MutatorStmtCall::transform)),
            _ => panic!("unknown transformer {}", transformer_name),
        }
    }

    // this funciton gives a vec of all transformers, in order they are executed
    pub fn all_transformers() -> Vec<String> {
        [
            "lit_int",
            "lit_bool",
            "unop_not",
            "binop_add",
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
