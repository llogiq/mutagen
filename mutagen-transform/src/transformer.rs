use proc_macro2::Span;
use quote::ToTokens;
use syn::fold::Fold;
use syn::{Expr, ItemFn};

mod transformer_binop_add;
mod transformer_lit_bool;
mod transformer_lit_int;
mod transformer_unop_not;

use transformer_binop_add::MutagenTransformerBinopAdd;
use transformer_lit_bool::MutagenTransformerLitBool;
use transformer_lit_int::MutagenTransformerLitInt;
use transformer_unop_not::MutagenTransformerUnopNot;

use crate::args::arg_options::Transformers;
use crate::transform_info::SharedTransformInfo;

pub enum MutagenTransformer {
    Expr(Box<dyn MutagenExprTransformer>),
}

pub struct MutagenTransformerBundle {
    pub expr_transformers: Vec<Box<dyn MutagenExprTransformer>>,
}

/// trait that is implemented by all transformers.
///
/// each transformer should not inspect the expression recursively since recursion is performed by the `MutagenTransformerBundle`
pub trait MutagenExprTransformer {
    fn map_expr(&mut self, expr: Expr) -> ExprTransformerOutput;
}

pub enum ExprTransformerOutput {
    Transformed(TransformedExpr),
    Unchanged(Expr),
}

/// An Expr that has been transformed.
///
/// This struct also contains the span of the original code for further processing
pub struct TransformedExpr {
    expr: Expr,
    span: Span,
}

impl ExprTransformerOutput {
    pub fn unchanged(expr: Expr) -> Self {
        ExprTransformerOutput::Unchanged(expr)
    }

    pub fn changed(expr: Expr, span: Span) -> Self {
        ExprTransformerOutput::Transformed(TransformedExpr { expr, span })
    }
}

impl Fold for MutagenTransformerBundle {
    fn fold_expr(&mut self, e: Expr) -> Expr {
        // transform content of the expression first
        let mut result = match e {
            Expr::Box(e0) => Expr::Box(self.fold_expr_box(e0)),
            Expr::InPlace(e0) => Expr::InPlace(self.fold_expr_in_place(e0)),
            Expr::Array(e0) => Expr::Array(self.fold_expr_array(e0)),
            Expr::Call(e0) => Expr::Call(self.fold_expr_call(e0)),
            Expr::MethodCall(e0) => Expr::MethodCall(self.fold_expr_method_call(e0)),
            Expr::Tuple(e0) => Expr::Tuple(self.fold_expr_tuple(e0)),
            Expr::Binary(e0) => Expr::Binary(self.fold_expr_binary(e0)),
            Expr::Unary(e0) => Expr::Unary(self.fold_expr_unary(e0)),
            Expr::Lit(e0) => Expr::Lit(self.fold_expr_lit(e0)),
            Expr::Cast(e0) => Expr::Cast(self.fold_expr_cast(e0)),
            Expr::Type(e0) => Expr::Type(self.fold_expr_type(e0)),
            Expr::Let(e0) => Expr::Let(self.fold_expr_let(e0)),
            Expr::If(e0) => Expr::If(self.fold_expr_if(e0)),
            Expr::While(e0) => Expr::While(self.fold_expr_while(e0)),
            Expr::ForLoop(e0) => Expr::ForLoop(self.fold_expr_for_loop(e0)),
            Expr::Loop(e0) => Expr::Loop(self.fold_expr_loop(e0)),
            Expr::Match(e0) => Expr::Match(self.fold_expr_match(e0)),
            Expr::Closure(e0) => Expr::Closure(self.fold_expr_closure(e0)),
            Expr::Unsafe(e0) => Expr::Unsafe(self.fold_expr_unsafe(e0)),
            Expr::Block(e0) => Expr::Block(self.fold_expr_block(e0)),
            Expr::Assign(e0) => Expr::Assign(self.fold_expr_assign(e0)),
            Expr::AssignOp(e0) => Expr::AssignOp(self.fold_expr_assign_op(e0)),
            Expr::Field(e0) => Expr::Field(self.fold_expr_field(e0)),
            Expr::Index(e0) => Expr::Index(self.fold_expr_index(e0)),
            Expr::Range(e0) => Expr::Range(self.fold_expr_range(e0)),
            Expr::Path(e0) => Expr::Path(self.fold_expr_path(e0)),
            Expr::Reference(e0) => Expr::Reference(self.fold_expr_reference(e0)),
            Expr::Break(e0) => Expr::Break(self.fold_expr_break(e0)),
            Expr::Continue(e0) => Expr::Continue(self.fold_expr_continue(e0)),
            Expr::Return(e0) => Expr::Return(self.fold_expr_return(e0)),
            Expr::Macro(e0) => Expr::Macro(self.fold_expr_macro(e0)),
            Expr::Struct(e0) => Expr::Struct(self.fold_expr_struct(e0)),
            Expr::Repeat(e0) => Expr::Repeat(self.fold_expr_repeat(e0)),
            Expr::Paren(e0) => Expr::Paren(self.fold_expr_paren(e0)),
            Expr::Group(e0) => Expr::Group(self.fold_expr_group(e0)),
            Expr::Try(e0) => Expr::Try(self.fold_expr_try(e0)),
            Expr::Async(e0) => Expr::Async(self.fold_expr_async(e0)),
            Expr::TryBlock(e0) => Expr::TryBlock(self.fold_expr_try_block(e0)),
            Expr::Yield(e0) => Expr::Yield(self.fold_expr_yield(e0)),
            Expr::Verbatim(e0) => Expr::Verbatim(self.fold_expr_verbatim(e0)),
        };

        // call all transformers on this expression
        for t in &mut self.expr_transformers {
            match t.map_expr(result) {
                ExprTransformerOutput::Transformed(TransformedExpr { expr, span }) => {
                    let transformed = set_true_span::set_true_span(expr.into_token_stream(), span);
                    result = syn::parse2(transformed).unwrap()
                }
                ExprTransformerOutput::Unchanged(e) => {
                    result = e;
                }
            }
        }
        result
    }
}

impl MutagenTransformerBundle {
    pub fn mutagen_transform_item_fn(&mut self, target: ItemFn) -> ItemFn {
        self.fold_item_fn(target)
    }

    pub fn new(transformers: Transformers, transform_info: &SharedTransformInfo) -> Self {
        let transformers = match transformers {
            Transformers::All => all_transformers(),
            Transformers::Only(list) => {
                let mut transformers = list.transformers;
                transformers.sort_by_key(|t| TRANSFORMER_ORDER[t]);
                transformers
            }
            Transformers::Not(list) => {
                let mut transformers = all_transformers();
                for l in &list.transformers {
                    transformers.remove_item(l);
                }
                transformers
            }
        };

        let mut expr_transformers = Vec::new();
        for t in &transformers {
            let t = mk_transformer(t, &[], transform_info.clone_shared());
            match t {
                MutagenTransformer::Expr(t) => expr_transformers.push(t),
            }
        }

        Self { expr_transformers }
    }
}

fn mk_transformer(
    transformer_name: &str,
    _transformer_args: &[String],
    transform_info: SharedTransformInfo,
) -> MutagenTransformer {
    match transformer_name {
        "lit_int" => MutagenTransformer::Expr(box MutagenTransformerLitInt { transform_info }),
        "lit_bool" => MutagenTransformer::Expr(box MutagenTransformerLitBool { transform_info }),
        "unop_not" => MutagenTransformer::Expr(box MutagenTransformerUnopNot { transform_info }),
        "binop_add" => MutagenTransformer::Expr(box MutagenTransformerBinopAdd { transform_info }),
        _ => panic!("unknown transformer {}", transformer_name),
    }
}

// this funciton gives a vec of all transformers, in order they are executed
fn all_transformers() -> Vec<String> {
    ["lit_int", "lit_bool", "unop_not", "binop_add"]
        .iter()
        .copied()
        .map(ToOwned::to_owned)
        .collect()
}

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref TRANSFORMER_ORDER: HashMap<String, usize> = {
        all_transformers()
            .into_iter()
            .enumerate()
            .map(|(i, s)| (s, i))
            .collect()
    };
}

/// sets the span of the generated code to be at the location of the original code.
///
/// However, the flag `procmacro2_semver_exempt` is required. Otherwise the function `located_at` is not exported. It is required to call the test suite with `RUSTFLAGS='--cfg procmacro2_semver_exempt' cargo test` to enable that feature.
#[cfg(procmacro2_semver_exempt)]
mod set_true_span {

    use proc_macro2::{Group, Span, TokenStream, TokenTree};

    // replaces all occurences of the default span with the given one
    pub fn set_true_span(stream: TokenStream, new_span: Span) -> TokenStream {
        stream
            .into_iter()
            .map(|tt| {
                let mut tt = if let TokenTree::Group(g) = tt {
                    let new_stream = set_true_span(g.stream(), new_span);
                    TokenTree::Group(Group::new(g.delimiter(), new_stream))
                } else {
                    tt
                };
                let current_span = tt.span();
                if Span::call_site().eq(&current_span) {
                    // located_at is semver excempt
                    tt.set_span(current_span.located_at(new_span));
                } else {
                }
                tt
            })
            .collect()
    }

}

/// if the flag `procmacro2_semver_exempt` is not enabled, a dummy implementation is provided, which does not change the spans
#[cfg(not(procmacro2_semver_exempt))]
mod set_true_span {
    use proc_macro2::{Span, TokenStream};

    // replaces all occurences of the default span with the given one
    pub fn set_true_span(stream: TokenStream, _new_span: Span) -> TokenStream {
        stream
    }
}
