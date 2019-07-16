use syn::spanned::Spanned;
use syn::{parse_quote, BinOp, Expr, ExprBinary};

use mutagen_core::Mutation;

use super::{ExprTransformerOutput, MutagenExprTransformer};
use crate::transform_info::SharedTransformInfo;

pub struct MutagenTransformerBinopAdd {
    pub transform_info: SharedTransformInfo,
}

impl MutagenExprTransformer for MutagenTransformerBinopAdd {
    fn map_expr(&mut self, e: Expr) -> ExprTransformerOutput {
        match e {
            Expr::Binary(ExprBinary {
                left,
                right,
                op: BinOp::Add(op_add),
                ..
            }) => {
                let mutator_id = self
                    .transform_info
                    .add_mutation(Mutation::new_spanned("binop_add".to_owned(), op_add.span()));
                let expr = parse_quote! {
                    <::mutagen::mutator::MutatorBinopAdd<_, _>>
                        ::new(#mutator_id, #left, #right)
                        .run_mutator(
                            ::mutagen::MutagenRuntimeConfig::get_default()
                        )
                };
                ExprTransformerOutput::changed(expr, op_add.span())
            }
            _ => ExprTransformerOutput::unchanged(e),
        }
    }
}
