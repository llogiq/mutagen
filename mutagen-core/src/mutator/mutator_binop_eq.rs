//! Mutator for binary operations `==` and `!=`

use std::convert::TryFrom;
use std::ops::Deref;

use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::{BinOp, Expr, ExprBinary};

use crate::comm::Mutation;
use crate::transformer::TransformContext;
use crate::transformer::transform_info::SharedTransformInfo;

use crate::MutagenRuntimeConfig;

pub struct MutatorBinopEq {}

impl MutatorBinopEq {
    pub fn run<L: PartialEq<R>, R>(
        mutator_id: usize,
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

    pub fn transform(
        e: Expr,
        transform_info: &SharedTransformInfo,
        context: &TransformContext,
    ) -> Expr {
        let e = match ExprBinopEq::try_from(e) {
            Ok(e) => e,
            Err(e) => return e,
        };

        let mutator_id = transform_info.add_mutations(
            MutationBinopEq::possible_mutations(e.op)
                .iter()
                .map(|m| m.to_mutation(&e, context)),
        );

        let left = &e.left;
        let right = &e.right;
        let op = e.op_tokens();

        syn::parse2(quote_spanned! {e.span=>
            ::mutagen::mutator::MutatorBinopEq::run(
                    #mutator_id,
                    &(#left),
                    &(#right),
                    #op,
                    ::mutagen::MutagenRuntimeConfig::get_default()
                )
        })
        .expect("transformed code invalid")
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

    fn to_mutation(self, original_op: &ExprBinopEq, context: &TransformContext) -> Mutation {
        Mutation::new_spanned(
            &context,
            "binop_eq".to_owned(),
            format!("{}", original_op.op),
            format!("{}", self.op),
            original_op.span,
        )
    }
}

#[derive(Clone, Debug)]
struct ExprBinopEq {
    op: BinopEq,
    left: Expr,
    right: Expr,
    span: Span,
}

impl TryFrom<Expr> for ExprBinopEq {
    type Error = Expr;
    fn try_from(expr: Expr) -> Result<Self, Expr> {
        match expr {
            Expr::Binary(ExprBinary {
                left,
                right,
                op,
                attrs,
            }) => match op {
                BinOp::Eq(t) => Ok(ExprBinopEq {
                    op: BinopEq::Eq,
                    left: *left,
                    right: *right,
                    span: t.span(),
                }),
                BinOp::Ne(t) => Ok(ExprBinopEq {
                    op: BinopEq::Ne,
                    left: *left,
                    right: *right,
                    span: t.span(),
                }),
                _ => Err(Expr::Binary(ExprBinary {
                    left,
                    right,
                    op,
                    attrs,
                })),
            },
            _ => Err(expr),
        }
    }
}

impl ExprBinopEq {
    fn op_tokens(&self) -> TokenStream {
        let mut tokens = TokenStream::new();
        tokens.extend(quote_spanned!(self.span=>
                ::mutagen::mutator::mutator_binop_eq::BinopEq::));
        tokens.extend(match self.op {
            BinopEq::Eq => quote_spanned!(self.span=> Eq),
            BinopEq::Ne => quote_spanned!(self.span=> Ne),
        });
        tokens
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
