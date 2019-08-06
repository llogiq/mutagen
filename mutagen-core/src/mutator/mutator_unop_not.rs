//! Mutator for binary operation `+`.

use std::ops::Not;

use syn::spanned::Spanned;
use syn::{parse_quote, Expr, ExprUnary, UnOp};

use crate::transformer::transform_info::SharedTransformInfo;
use crate::transformer::ExprTransformerOutput;
use crate::Mutation;

use crate::optimistic::NotToNone;
use crate::MutagenRuntimeConfig;

pub struct MutatorUnopNot {}

impl MutatorUnopNot {
    pub fn run<T: Not>(
        mutator_id: u32,
        val: T,
        runtime: MutagenRuntimeConfig,
    ) -> <T as Not>::Output {
        if runtime.mutation_id != mutator_id {
            !val
        } else {
            val.may_none()
        }
    }

    pub fn transform(e: Expr, transform_info: &SharedTransformInfo) -> ExprTransformerOutput {
        match e {
            Expr::Unary(ExprUnary {
                expr,
                op: UnOp::Not(op_not),
                ..
            }) => {
                let mutator_id = transform_info.add_mutation(Mutation::new_spanned(
                    "unop_not".to_owned(),
                    "!".to_owned(),
                    "".to_owned(),
                    op_not.span(),
                ));
                let expr = parse_quote! {
                    ::mutagen::mutator::MutatorUnopNot::run(
                            #mutator_id,
                            #expr,
                            ::mutagen::MutagenRuntimeConfig::get_default()
                        )
                };
                ExprTransformerOutput::changed(expr, op_not.span())
            }
            _ => ExprTransformerOutput::unchanged(e),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn boolnot_inactive() {
        // input is true, but will be negated by non-active mutator
        let result = MutatorUnopNot::run(1, true, MutagenRuntimeConfig::with_mutation_id(0));
        assert_eq!(result, false);
    }
    #[test]
    fn boolnot_active() {
        let result = MutatorUnopNot::run(1, true, MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, true);
    }
    #[test]
    fn intnot_active() {
        let result = MutatorUnopNot::run(1, 1, MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, 1);
    }

    pub use crate::optimistic::{TypeWithNotOtherOutput, TypeWithNotTarget};

    #[test]
    fn optimistic_incorrect_inactive() {
        let result = MutatorUnopNot::run(
            1,
            TypeWithNotOtherOutput(),
            MutagenRuntimeConfig::with_mutation_id(0),
        );
        assert_eq!(result, TypeWithNotTarget());
    }
    #[test]
    #[should_panic]
    fn optimistic_incorrect_active() {
        MutatorUnopNot::run(
            1,
            TypeWithNotOtherOutput(),
            MutagenRuntimeConfig::with_mutation_id(1),
        );
    }

}
