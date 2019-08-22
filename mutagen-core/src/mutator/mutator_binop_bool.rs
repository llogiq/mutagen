//! Mutator for binary operations `&&` and `&&`.

use std::ops::Deref;

use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{BinOp, Expr, ExprBinary};

use crate::comm::Mutation;
use crate::transformer::transform_info::SharedTransformInfo;

use crate::MutagenRuntimeConfig;

pub struct MutatorBinopBool {}

impl MutatorBinopBool {
    pub fn run_left(
        mutator_id: usize,
        original_op: BinopBool,
        left: bool,
        runtime: impl Deref<Target = MutagenRuntimeConfig>,
    ) -> Option<bool> {
        runtime.covered(mutator_id);
        let mutations = MutationBinopBool::possible_mutations(original_op);
        let op = runtime
            .get_mutation(mutator_id, &mutations)
            .map(|m| m.op)
            .unwrap_or(original_op);
        op.short_circuit_left(left)
    }

    pub fn transform(e: Expr, transform_info: &SharedTransformInfo) -> Expr {
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
                        return Expr::Binary(ExprBinary {
                            left,
                            right,
                            op,
                            attrs,
                        })
                    }
                };

                let mutator_id = transform_info.add_mutations(
                    MutationBinopBool::possible_mutations(op)
                        .iter()
                        .map(|m| m.to_mutation(op, tt.span())),
                );

                syn::parse2(quote_spanned! {op.span()=>
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
                })
                .expect("transformed code invalid")
            }
            _ => e,
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
mod tests {

    use super::*;

    #[test]
    fn possible_mutations_and() {
        assert_eq!(
            MutationBinopBool::possible_mutations(BinopBool::And),
            vec![MutationBinopBool { op: BinopBool::Or }]
        )
    }

    #[test]
    fn possible_mutations_or() {
        assert_eq!(
            MutationBinopBool::possible_mutations(BinopBool::Or),
            vec![MutationBinopBool { op: BinopBool::And }]
        )
    }

    #[test]
    fn short_circuit_left_and_false() {
        assert_eq!(BinopBool::And.short_circuit_left(false), Some(false))
    }
    #[test]
    fn short_circuit_left_and_true() {
        assert_eq!(BinopBool::And.short_circuit_left(true), None)
    }

    #[test]
    fn short_circuit_left_or_false() {
        assert_eq!(BinopBool::Or.short_circuit_left(false), None)
    }
    #[test]
    fn short_circuit_left_or_true() {
        assert_eq!(BinopBool::Or.short_circuit_left(true), Some(true))
    }

    #[test]
    fn mutator_and_inactive() {
        assert_eq!(
            MutatorBinopBool::run_left(
                1,
                BinopBool::And,
                true,
                &MutagenRuntimeConfig::without_mutation()
            ),
            None
        );
        assert_eq!(
            MutatorBinopBool::run_left(
                1,
                BinopBool::And,
                false,
                &MutagenRuntimeConfig::without_mutation()
            ),
            Some(false)
        );
    }
    #[test]
    fn mutator_and_active() {
        assert_eq!(
            MutatorBinopBool::run_left(
                1,
                BinopBool::And,
                true,
                &MutagenRuntimeConfig::with_mutation_id(1)
            ),
            Some(true)
        );
        assert_eq!(
            MutatorBinopBool::run_left(
                1,
                BinopBool::And,
                false,
                &MutagenRuntimeConfig::with_mutation_id(1)
            ),
            None
        );
    }

    #[test]
    fn mutator_or_inactive() {
        assert_eq!(
            MutatorBinopBool::run_left(
                1,
                BinopBool::Or,
                true,
                &MutagenRuntimeConfig::without_mutation()
            ),
            Some(true)
        );
        assert_eq!(
            MutatorBinopBool::run_left(
                1,
                BinopBool::Or,
                false,
                &MutagenRuntimeConfig::without_mutation()
            ),
            None
        );
    }
    #[test]
    fn mutator_or_active() {
        assert_eq!(
            MutatorBinopBool::run_left(
                1,
                BinopBool::Or,
                true,
                &MutagenRuntimeConfig::with_mutation_id(1)
            ),
            None
        );
        assert_eq!(
            MutatorBinopBool::run_left(
                1,
                BinopBool::Or,
                false,
                &MutagenRuntimeConfig::with_mutation_id(1)
            ),
            Some(false)
        );
    }
}
