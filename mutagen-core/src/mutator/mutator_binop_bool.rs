//! Mutator for binary operations `&&` and `&&`.

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_quote, BinOp, Expr, ExprBinary};

use crate::transformer::transform_info::SharedTransformInfo;
use crate::transformer::ExprTransformerOutput;
use crate::Mutation;

use crate::MutagenRuntimeConfig;

pub struct MutatorBinopBool {}

impl MutatorBinopBool {
    pub fn run_left(
        mutator_id: u32,
        original_op: BinopBool,
        left: bool,
        runtime: MutagenRuntimeConfig,
    ) -> Option<bool> {
        let mutations = MutationBinopBool::possible_mutations(original_op);
        let op = runtime
            .get_mutation(mutator_id, &mutations)
            .map(|m| m.op)
            .unwrap_or(original_op);
        op.short_circuit_left(left)
    }

    pub fn transform(e: Expr, transform_info: &SharedTransformInfo) -> ExprTransformerOutput {
        match e {
            Expr::Binary(ExprBinary {
                left,
                right,
                op,
                attrs,
            }) => {
                let (op, tt) = match op {
                    BinOp::And(t) => (BinopBool::And, t.into_token_stream()),
                    BinOp::Or(t) => (BinopBool::Or, t.into_token_stream()),
                    _ => {
                        return ExprTransformerOutput::unchanged(Expr::Binary(ExprBinary {
                            left,
                            right,
                            op,
                            attrs,
                        }))
                    }
                };

                let mutator_id = transform_info.add_mutations(
                    MutationBinopBool::possible_mutations(op)
                        .iter()
                        .map(|m| m.to_mutation(op, tt.span())),
                );

                let expr = parse_quote! {
                    if let Some(x) = ::mutagen::mutator::MutatorBinopBool::run_left(
                            #mutator_id,
                            #op,
                            #left,
                            ::mutagen::MutagenRuntimeConfig::get_default()
                        ) {
                        x
                    } else {
                        #right
                    }
                };
                ExprTransformerOutput::changed(expr, tt.span())
            }
            _ => ExprTransformerOutput::unchanged(e),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct MutationBinopBool {
    op: BinopBool,
}

impl MutationBinopBool {
    fn possible_mutations(original_op: BinopBool) -> Vec<Self> {
        [BinopBool::And, BinopBool::Or]
            .iter()
            .copied()
            .filter(|&op| op != original_op)
            .map(|op| MutationBinopBool { op })
            .collect()
    }

    fn to_mutation(self, original_op: BinopBool, span: Span) -> Mutation {
        Mutation::new_spanned(
            "binop_bool".to_owned(),
            format!("{}", original_op),
            format!("{}", self.op),
            span,
        )
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BinopBool {
    And,
    Or,
}

impl BinopBool {
    pub fn short_circuit_left(self, left: bool) -> Option<bool> {
        match self {
            BinopBool::And if !left => Some(false),
            BinopBool::Or if left => Some(true),
            _ => None,
        }
    }
}

impl ToTokens for BinopBool {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote!(::mutagen::mutator::mutator_binop_bool::BinopBool::));
        tokens.extend(match self {
            BinopBool::And => quote!(And),
            BinopBool::Or => quote!(Or),
        })
    }
}

use std::fmt;

impl fmt::Display for BinopBool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BinopBool::And => write!(f, "&&"),
            BinopBool::Or => write!(f, "||"),
        }
    }
}

#[cfg(test)]
mod tests {}
