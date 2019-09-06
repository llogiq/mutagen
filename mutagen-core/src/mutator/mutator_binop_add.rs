//! Mutator for binary operation `+`.

use std::convert::TryFrom;
use std::ops::Add;
use std::ops::Deref;

use crate::transformer::ast_inspect::ExprBinopAdd;
use quote::quote_spanned;
use syn::Expr;

use crate::comm::Mutation;
use crate::transformer::transform_info::SharedTransformInfo;
use crate::transformer::TransformContext;

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

    pub fn run_native_num<I: Add<I, Output = I>>(
        mutator_id: usize,
        left: I,
        right: I,
        runtime: impl Deref<Target = MutagenRuntimeConfig>,
    ) -> I {
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
            &context,
            "binop_add".to_owned(),
            "+".to_owned(),
            "-".to_owned(),
            e.span,
        ));

        let left = &e.left;
        let right = &e.right;

        // if the current expression is based on numbers, use the function `run_native_num` instead
        let run_fn = if context.is_num_expr() {
            quote_spanned! {e.span=> run_native_num}
        } else {
            quote_spanned! {e.span=> run}
        };

        syn::parse2(quote_spanned! {e.span=>
            ::mutagen::mutator::MutatorBinopAdd::#run_fn(
                    #mutator_id,
                    #left,
                    #right,
                    ::mutagen::MutagenRuntimeConfig::get_default()
                )
        })
        .expect("transformed code invalid")
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

    // TODO: tests for native_num
}
