//! Mutator for binary operations `&&` and `||`.

use std::convert::TryFrom;
use std::ops::Deref;

use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::{BinOp, Expr};

use crate::comm::Mutation;
use crate::transformer::transform_info::SharedTransformInfo;
use crate::transformer::TransformContext;

use crate::MutagenRuntimeConfig;

pub fn run_left(
    mutator_id: usize,
    original_op: BinopBool,
    left: bool,
    runtime: impl Deref<Target = MutagenRuntimeConfig>,
) -> Option<bool> {
    runtime.covered(mutator_id);
    let mutations = MutationBinopBool::possible_mutations(original_op);
    let op = runtime
        .get_mutation_for_mutator(mutator_id, &mutations)
        .map(|m| m.op)
        .unwrap_or(original_op);
    op.short_circuit_left(left)
}

pub fn transform(
    e: Expr,
    transform_info: &SharedTransformInfo,
    context: &TransformContext,
) -> Expr {
    let e = match ExprBinopBool::try_from(e) {
        Ok(e) => e,
        Err(e) => return e,
    };

    let mutator_id = transform_info.add_mutations(
        MutationBinopBool::possible_mutations(e.op)
            .iter()
            .map(|m| m.to_mutation(&e, context)),
    );

    let left = &e.left;
    let right = &e.right;
    let op = e.op_tokens();

    syn::parse2(quote_spanned! {e.span=>
        if let Some(x) = ::mutagen::mutator::mutator_binop_bool::run_left(
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

    fn to_mutation(self, original_op: &ExprBinopBool, context: &TransformContext) -> Mutation {
        Mutation::new_spanned(
            &context,
            "binop_bool".to_owned(),
            format!("{}", original_op),
            format!("{}", self.op),
            original_op.span,
        )
    }
}

#[derive(Clone, Debug)]
struct ExprBinopBool {
    op: BinopBool,
    left: Expr,
    right: Expr,
    span: Span,
}

impl TryFrom<Expr> for ExprBinopBool {
    type Error = Expr;
    fn try_from(expr: Expr) -> Result<Self, Expr> {
        match expr {
            Expr::Binary(expr) => match expr.op {
                BinOp::And(t) => Ok(ExprBinopBool {
                    op: BinopBool::And,
                    left: *expr.left,
                    right: *expr.right,
                    span: t.span(),
                }),
                BinOp::Or(t) => Ok(ExprBinopBool {
                    op: BinopBool::Or,
                    left: *expr.left,
                    right: *expr.right,
                    span: t.span(),
                }),
                _ => Err(Expr::Binary(expr)),
            },
            _ => Err(expr),
        }
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

impl ExprBinopBool {
    fn op_tokens(&self) -> TokenStream {
        let mut tokens = TokenStream::new();
        tokens.extend(quote_spanned!(self.span=>
            ::mutagen::mutator::mutator_binop_bool::BinopBool::));
        tokens.extend(match self.op {
            BinopBool::And => quote_spanned!(self.span=> And),
            BinopBool::Or => quote_spanned!(self.span=> Or),
        });
        tokens
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

impl fmt::Display for ExprBinopBool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.op)
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
            run_left(
                1,
                BinopBool::And,
                true,
                &MutagenRuntimeConfig::without_mutation()
            ),
            None
        );
        assert_eq!(
            run_left(
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
            run_left(
                1,
                BinopBool::And,
                true,
                &MutagenRuntimeConfig::with_mutation_id(1)
            ),
            Some(true)
        );
        assert_eq!(
            run_left(
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
            run_left(
                1,
                BinopBool::Or,
                true,
                &MutagenRuntimeConfig::without_mutation()
            ),
            Some(true)
        );
        assert_eq!(
            run_left(
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
            run_left(
                1,
                BinopBool::Or,
                true,
                &MutagenRuntimeConfig::with_mutation_id(1)
            ),
            None
        );
        assert_eq!(
            run_left(
                1,
                BinopBool::Or,
                false,
                &MutagenRuntimeConfig::with_mutation_id(1)
            ),
            Some(false)
        );
    }
}
