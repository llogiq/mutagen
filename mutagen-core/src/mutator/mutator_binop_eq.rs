//! Mutator for binary operation `==`.

use std::cmp::PartialEq;

use syn::spanned::Spanned;
use syn::{parse_quote, BinOp, Expr, ExprBinary};

use crate::transform_info::SharedTransformInfo;
use crate::transformer::ExprTransformerOutput;
use crate::Mutation;

use crate::MutagenRuntimeConfig;

pub struct MutatorBinopEq {}

impl MutatorBinopEq {
    pub fn run<L: PartialEq<R>, R>(
        mutator_id: u32,
        left: L,
        right: R,
        runtime: MutagenRuntimeConfig,
    ) -> bool {
        if runtime.mutation_id != mutator_id {
            left == right
        } else {
            left != right
        }
    }

    pub fn transform(e: Expr, transform_info: &SharedTransformInfo) -> ExprTransformerOutput {
        match e {
            Expr::Binary(ExprBinary {
                left,
                right,
                op: BinOp::Eq(op_eq),
                ..
            }) => {
                let mutator_id = transform_info.add_mutation(Mutation::new_spanned(
                    "binop_eq".to_owned(),
                    "replcae `==` with `!=`".to_owned(),
                    op_eq.span(),
                ));
                let expr = parse_quote! {
                    ::mutagen::mutator::MutatorBinopEq::run::<_, _>(
                            #mutator_id,
                            #left,
                            #right,
                            ::mutagen::MutagenRuntimeConfig::get_default()
                        )
                };
                ExprTransformerOutput::changed(expr, op_eq.span())
            }
            _ => ExprTransformerOutput::unchanged(e),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn eq_inactive() {
        let result = MutatorBinopEq::run(1, 5, 4, MutagenRuntimeConfig::with_mutation_id(0));
        assert_eq!(result, false);
    }
    #[test]
    fn eq_active() {
        let result = MutatorBinopEq::run(1, 5, 4, MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, true);
    }

}
