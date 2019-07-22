//! Mutator for boolean literals.

use syn::{parse_quote, Expr, ExprLit, Lit, LitBool};

use crate::transform_info::SharedTransformInfo;
use crate::transformer::ExprTransformerOutput;
use crate::Mutation;

use crate::MutagenRuntimeConfig;

pub struct MutatorLitBool {}

impl MutatorLitBool {
    pub fn run(mutator_id: u32, original_lit: bool, runtime: MutagenRuntimeConfig) -> bool {
        if runtime.mutation_id != mutator_id {
            original_lit
        } else {
            !original_lit
        }
    }

    pub fn transform(e: Expr, transform_info: &SharedTransformInfo) -> ExprTransformerOutput {
        match e {
            Expr::Lit(ExprLit {
                lit: Lit::Bool(LitBool { value, span }),
                ..
            }) => {
                let mutator_id = transform_info.add_mutation(Mutation::new_spanned(
                    "lit_bool".to_owned(),
                    format!("replace {:?} with {:?}", value, !value),
                    span,
                ));
                let expr = parse_quote! {
                    ::mutagen::mutator::MutatorLitBool::run(
                            #mutator_id,
                            #value,
                            ::mutagen::MutagenRuntimeConfig::get_default()
                        )
                };
                ExprTransformerOutput::changed(expr, span)
            }
            _ => ExprTransformerOutput::unchanged(e),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::MutagenRuntimeConfig;

    #[test]
    pub fn false_inactive() {
        let result = MutatorLitBool::run(1, false, MutagenRuntimeConfig::with_mutation_id(0));
        assert_eq!(result, false)
    }
    #[test]
    pub fn true_inactive() {
        let result = MutatorLitBool::run(1, true, MutagenRuntimeConfig::with_mutation_id(0));
        assert_eq!(result, true)
    }
    #[test]
    pub fn false_active() {
        let result = MutatorLitBool::run(1, false, MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, true)
    }
    #[test]
    pub fn true_active() {
        let result = MutatorLitBool::run(1, true, MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, false)
    }

}
