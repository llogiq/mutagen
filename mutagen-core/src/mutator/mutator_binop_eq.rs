//! Mutator for binary operations `==` and `!=`

use std::ops::Deref;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_quote, BinOp, Expr, ExprBinary};

use crate::transformer::transform_info::SharedTransformInfo;
use crate::transformer::ExprTransformerOutput;
use crate::Mutation;

use crate::MutagenRuntimeConfig;

pub struct MutatorBinopEq {}

impl MutatorBinopEq {
    pub fn run<L: PartialEq<R>, R>(
        mutator_id: u32,
        left: L,
        right: R,
        original_op: BinopEq,
        runtime: impl Deref<Target = MutagenRuntimeConfig>,
    ) -> bool {
        runtime.covered(mutator_id);
        let mutations = MutationBinopEq::possible_mutations(original_op);
        if let Some(m) = runtime.get_mutation(mutator_id, &mutations) {
            m.mutate(left, right)
        } else {
            original_op.eq(left, right)
        }
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
                    BinOp::Eq(t) => (BinopEq::Eq, t.into_token_stream()),
                    BinOp::Ne(t) => (BinopEq::Ne, t.into_token_stream()),
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
                    MutationBinopEq::possible_mutations(op)
                        .iter()
                        .map(|m| m.to_mutation(op, tt.span())),
                );

                let expr = parse_quote! {
                    ::mutagen::mutator::MutatorBinopEq::run(
                            #mutator_id,
                            #left,
                            #right,
                            #op,
                            ::mutagen::MutagenRuntimeConfig::get_default()
                        )
                };
                ExprTransformerOutput::changed(expr, tt.span())
            }
            _ => ExprTransformerOutput::unchanged(e),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct MutationBinopEq {
    op: BinopEq,
}

impl MutationBinopEq {
    fn possible_mutations(original_op: BinopEq) -> Vec<Self> {
        [BinopEq::Eq, BinopEq::Ne]
            .iter()
            .copied()
            .filter(|&op| op != original_op)
            .map(|op| MutationBinopEq { op })
            .collect()
    }

    fn mutate<L: PartialEq<R>, R>(self, left: L, right: R) -> bool {
        self.op.eq(left, right)
    }

    fn to_mutation(self, original_op: BinopEq, span: Span) -> Mutation {
        Mutation::new_spanned(
            "binop_eq".to_owned(),
            format!("{}", original_op),
            format!("{}", self.op),
            span,
        )
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BinopEq {
    Eq,
    Ne,
}

impl BinopEq {
    fn eq<L: PartialEq<R>, R>(self, left: L, right: R) -> bool {
        match self {
            BinopEq::Eq => left == right,
            BinopEq::Ne => left != right,
        }
    }
}

impl ToTokens for BinopEq {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote!(::mutagen::mutator::mutator_binop_eq::BinopEq::));
        tokens.extend(match self {
            BinopEq::Eq => quote!(Eq),
            BinopEq::Ne => quote!(Ne),
        })
    }
}

use std::fmt;

impl fmt::Display for BinopEq {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BinopEq::Eq => write!(f, "=="),
            BinopEq::Ne => write!(f, "!="),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn eq_inactive() {
        let result = MutatorBinopEq::run(
            1,
            5,
            4,
            BinopEq::Eq,
            &MutagenRuntimeConfig::without_mutation(),
        );
        assert_eq!(result, false);
    }
    #[test]
    fn eq_active() {
        let result = MutatorBinopEq::run(
            1,
            5,
            4,
            BinopEq::Eq,
            &MutagenRuntimeConfig::with_mutation_id(1),
        );
        assert_eq!(result, true);
    }

    #[test]
    fn ne_inactive() {
        let result = MutatorBinopEq::run(
            1,
            5,
            4,
            BinopEq::Ne,
            &MutagenRuntimeConfig::without_mutation(),
        );
        assert_eq!(result, true);
    }
    #[test]
    fn ne_active() {
        let result = MutatorBinopEq::run(
            1,
            5,
            4,
            BinopEq::Ne,
            &MutagenRuntimeConfig::with_mutation_id(1),
        );
        assert_eq!(result, false);
    }
}
