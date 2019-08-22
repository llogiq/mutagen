//! Mutator for boolean literals.

use std::ops::Deref;

use quote::quote_spanned;
use syn::{Expr, ExprLit, Lit, LitBool};

use crate::comm::Mutation;
use crate::transformer::transform_info::SharedTransformInfo;

use crate::MutagenRuntimeConfig;

pub struct MutatorLitBool {}

impl MutatorLitBool {
    pub fn run(
        mutator_id: usize,
        original_lit: bool,
        runtime: impl Deref<Target = MutagenRuntimeConfig>,
    ) -> bool {
        runtime.covered(mutator_id);
        if runtime.is_mutation_active(mutator_id) {
            !original_lit
        } else {
            original_lit
        }
    }

    pub fn transform(e: Expr, transform_info: &SharedTransformInfo) -> Expr {
        match e {
            Expr::Lit(ExprLit {
                lit: Lit::Bool(LitBool { value, span }),
                ..
            }) => {
                let mutator_id = transform_info.add_mutation(Mutation::new_spanned(
                    "lit_bool".to_owned(),
                    format!("{:?}", value),
                    format!("{:?}", !value),
                    span,
                ));
                syn::parse2(quote_spanned! {span=>
                    ::mutagen::mutator::MutatorLitBool::run(
                            #mutator_id,
                            #value,
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
    use crate::MutagenRuntimeConfig;

    #[test]
    pub fn false_inactive() {
        let result = MutatorLitBool::run(1, false, &MutagenRuntimeConfig::without_mutation());
        assert_eq!(result, false)
    }
    #[test]
    pub fn true_inactive() {
        let result = MutatorLitBool::run(1, true, &MutagenRuntimeConfig::without_mutation());
        assert_eq!(result, true)
    }
    #[test]
    pub fn false_active() {
        let result = MutatorLitBool::run(1, false, &MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, true)
    }
    #[test]
    pub fn true_active() {
        let result = MutatorLitBool::run(1, true, &MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, false)
    }
}
