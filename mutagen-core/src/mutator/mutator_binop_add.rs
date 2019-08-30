//! Mutator for binary operation `+`.

use std::convert::TryFrom;
use std::ops::Add;
use std::ops::Deref;

use proc_macro2::Span;

use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::{BinOp, Expr, ExprBinary};

use crate::comm::Mutation;
use crate::transformer::transform_context::TransformContext;
use crate::transformer::transform_info::SharedTransformInfo;

use crate::optimistic::AddToSub;
use crate::MutagenRuntimeConfig;

pub struct MutatorBinopAdd {}

impl MutatorBinopAdd {
    pub fn run<L: Add<R>, R>(
        mutator_id: usize,
        left: L,
        right: R,
        runtime: impl Deref<Target = MutagenRuntimeConfig>,
    ) -> <L as Add<R>>::Output {
        runtime.covered(mutator_id);
        if runtime.is_mutation_active(mutator_id) {
            left.may_sub(right)
        } else {
            left + right
        }
    }

    pub fn transform(
        e: Expr,
        transform_info: &SharedTransformInfo,
        context: &TransformContext,
    ) -> Expr {
        let e = match ExprBinopAdd::try_from(e) {
            Ok(e) => e,
            Err(e) => return e,
        };

        let mutator_id = transform_info.add_mutation(Mutation::new_spanned(
            context.fn_name.clone(),
            "binop_add".to_owned(),
            "+".to_owned(),
            "-".to_owned(),
            e.span,
        ));

        let left = &e.left;
        let right = &e.right;

        syn::parse2(quote_spanned! {e.span=>
            ::mutagen::mutator::MutatorBinopAdd::run(
                    #mutator_id,
                    #left,
                    #right,
                    ::mutagen::MutagenRuntimeConfig::get_default()
                )
        })
        .expect("transformed code invalid")
    }
}

#[derive(Clone, Debug)]
struct ExprBinopAdd {
    left: Expr,
    right: Expr,
    span: Span,
}

impl TryFrom<Expr> for ExprBinopAdd {
    type Error = Expr;
    fn try_from(expr: Expr) -> Result<Self, Expr> {
        match expr {
            Expr::Binary(ExprBinary {
                left,
                right,
                op,
                attrs,
            }) => match op {
                BinOp::Add(t) => Ok(ExprBinopAdd {
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn sum_inative() {
        let result = MutatorBinopAdd::run(1, 5, 4, &MutagenRuntimeConfig::without_mutation());
        assert_eq!(result, 9);
    }
    #[test]
    fn sum_ative() {
        let result = MutatorBinopAdd::run(1, 5, 4, &MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, 1);
    }

    #[test]
    fn str_add_inactive() {
        let result = MutatorBinopAdd::run(
            1,
            "x".to_string(),
            "y",
            &MutagenRuntimeConfig::without_mutation(),
        );
        assert_eq!(&result, "xy");
    }
    #[test]
    #[should_panic]
    fn str_add_active() {
        MutatorBinopAdd::run(
            1,
            "x".to_string(),
            "y",
            &MutagenRuntimeConfig::with_mutation_id(1),
        );
    }
}
