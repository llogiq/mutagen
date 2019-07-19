use syn::spanned::Spanned;
use syn::{parse_quote, BinOp, Expr, ExprBinary};

use crate::Mutation;

use super::{ExprTransformerOutput, MutagenExprTransformer};
use crate::transform_info::SharedTransformInfo;

pub struct MutagenTransformerBinopEq {
    pub transform_info: SharedTransformInfo,
}

impl MutagenExprTransformer for MutagenTransformerBinopEq {
    fn map_expr(&mut self, e: Expr) -> ExprTransformerOutput {
        match e {
            Expr::Binary(ExprBinary {
                left,
                right,
                op: BinOp::Eq(op_eq),
                ..
            }) => {
                let mutator_id = self
                    .transform_info
                    .add_mutation(Mutation::new_spanned("binop_eq".to_owned(), op_eq.span()));
                let expr = parse_quote! {
                    <::mutagen::mutator::MutatorBinopEq<_, _>>
                        ::new(#mutator_id, #left, #right)
                        .run_mutator(
                            ::mutagen::MutagenRuntimeConfig::get_default()
                        )
                };
                ExprTransformerOutput::changed(expr, op_eq.span())
            }
            _ => ExprTransformerOutput::unchanged(e),
        }
    }
}
