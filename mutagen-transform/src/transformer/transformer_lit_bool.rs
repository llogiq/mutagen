use syn::{parse_quote, Expr, ExprLit, Lit, LitBool};

use mutagen_core::Mutation;

use super::{ExprTransformerOutput, MutagenExprTransformer};
use crate::transform_info::SharedTransformInfo;

pub struct MutagenTransformerLitBool {
    pub transform_info: SharedTransformInfo,
}

// transforms bool literals to mutator expressions
impl MutagenExprTransformer for MutagenTransformerLitBool {
    fn map_expr(&mut self, e: Expr) -> ExprTransformerOutput {
        match e {
            Expr::Lit(ExprLit {
                lit: Lit::Bool(LitBool { value, span }),
                ..
            }) => {
                let mutator_id = self
                    .transform_info
                    .add_mutation(Mutation::new_spanned("lit_bool".to_owned(), span));
                let expr = parse_quote! {
                    ::mutagen::mutator::MutatorLitBool::new(#mutator_id, #value)
                        .run_mutator(
                            ::mutagen::MutagenRuntimeConfig::get_default()
                        )
                };
                ExprTransformerOutput::changed(expr, span)
            }
            _ => ExprTransformerOutput::unchanged(e),
        }
    }
}
