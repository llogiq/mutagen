//! Mutator for binary operation `+`.

use std::ops::Add;
use std::ops::Deref;

use syn::spanned::Spanned;
use syn::{parse_quote, BinOp, Expr, ExprBinary};

use crate::transformer::transform_info::SharedTransformInfo;
use crate::transformer::ExprTransformerOutput;
use crate::Mutation;

use crate::optimistic::AddToSub;
use crate::MutagenRuntimeConfig;

pub struct MutatorBinopAdd {}

impl MutatorBinopAdd {
    pub fn run<L: Add<R>, R>(
        mutator_id: u32,
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

    pub fn transform(e: Expr, transform_info: &SharedTransformInfo) -> ExprTransformerOutput {
        match e {
            Expr::Binary(ExprBinary {
                left,
                right,
                op: BinOp::Add(op_add),
                ..
            }) => {
                let mutator_id = transform_info.add_mutation(Mutation::new_spanned(
                    "binop_add".to_owned(),
                    "+".to_owned(),
                    "-".to_owned(),
                    op_add.span(),
                ));
                let expr = parse_quote! {
                    ::mutagen::mutator::MutatorBinopAdd::run(
                            #mutator_id,
                            #left,
                            #right,
                            ::mutagen::MutagenRuntimeConfig::get_default()
                        )
                };
                ExprTransformerOutput::changed(expr, op_add.span())
            }
            _ => ExprTransformerOutput::unchanged(e),
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
