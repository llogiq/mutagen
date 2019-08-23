//! Mutator for binary operation `+`.

use std::ops::Deref;
use std::ops::Not;

use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::{Expr, ExprUnary, UnOp};

use crate::comm::Mutation;
use crate::transformer::transform_context::TransformContext;
use crate::transformer::transform_info::SharedTransformInfo;

use crate::optimistic::NotToNone;
use crate::MutagenRuntimeConfig;

pub struct MutatorUnopNot {}

impl MutatorUnopNot {
    pub fn run<T: Not>(
        mutator_id: usize,
        val: T,
        runtime: impl Deref<Target = MutagenRuntimeConfig>,
    ) -> <T as Not>::Output {
        runtime.covered(mutator_id);
        if runtime.is_mutation_active(mutator_id) {
            val.may_none()
        } else {
            !val
        }
    }

    pub fn transform(
        e: Expr,
        transform_info: &SharedTransformInfo,
        context: &TransformContext,
    ) -> Expr {
        match e {
            Expr::Unary(ExprUnary {
                expr,
                op: UnOp::Not(op_not),
                ..
            }) => {
                let mutator_id = transform_info.add_mutation(Mutation::new_spanned(
                    context.fn_name.clone(),
                    "unop_not".to_owned(),
                    "!".to_owned(),
                    "".to_owned(),
                    op_not.span(),
                ));
                syn::parse2(quote_spanned! {op_not.span()=>
                    ::mutagen::mutator::MutatorUnopNot::run(
                            #mutator_id,
                            #expr,
                            ::mutagen::MutagenRuntimeConfig::get_default()
                        )
                })
                .expect("transformed code invalid")
            }
            _ => e,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn boolnot_inactive() {
        // input is true, but will be negated by non-active mutator
        let result = MutatorUnopNot::run(1, true, &MutagenRuntimeConfig::without_mutation());
        assert_eq!(result, false);
    }
    #[test]
    fn boolnot_active() {
        let result = MutatorUnopNot::run(1, true, &MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, true);
    }
    #[test]
    fn intnot_active() {
        let result = MutatorUnopNot::run(1, 1, &MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, 1);
    }

    pub use crate::optimistic::{TypeWithNotOtherOutput, TypeWithNotTarget};

    #[test]
    fn optimistic_incorrect_inactive() {
        let result = MutatorUnopNot::run(
            1,
            TypeWithNotOtherOutput(),
            &MutagenRuntimeConfig::without_mutation(),
        );
        assert_eq!(result, TypeWithNotTarget());
    }
    #[test]
    #[should_panic]
    fn optimistic_incorrect_active() {
        MutatorUnopNot::run(
            1,
            TypeWithNotOtherOutput(),
            &MutagenRuntimeConfig::with_mutation_id(1),
        );
    }
}
