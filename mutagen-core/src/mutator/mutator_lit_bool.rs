//! Mutator for boolean literals.

use std::convert::TryFrom;
use std::ops::Deref;

use proc_macro2::Span;
use quote::quote_spanned;
use syn::{Expr, ExprLit, Lit, LitBool};

use crate::comm::Mutation;
use crate::transformer::TransformContext;
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

    pub fn transform(
        e: Expr,
        transform_info: &SharedTransformInfo,
        context: &TransformContext,
    ) -> Expr {
        let e = match ExprLitBool::try_from(e) {
            Ok(e) => e,
            Err(e) => return e,
        };

        let mutator_id = transform_info.add_mutation(Mutation::new_spanned(
            &context,
            "lit_bool".to_owned(),
            format!("{:?}", e.value),
            format!("{:?}", !e.value),
            e.span,
        ));

        let value = e.value;
        syn::parse2(quote_spanned! {e.span=>
            ::mutagen::mutator::MutatorLitBool::run(
                    #mutator_id,
                    #value,
                    ::mutagen::MutagenRuntimeConfig::get_default()
                )
        })
        .expect("transformed code invalid")
    }
}

#[derive(Clone, Debug)]
struct ExprLitBool {
    value: bool,
    span: Span,
}

impl TryFrom<Expr> for ExprLitBool {
    type Error = Expr;
    fn try_from(expr: Expr) -> Result<Self, Expr> {
        match expr {
            Expr::Lit(ExprLit {
                lit: Lit::Bool(LitBool { value, span }),
                ..
            }) => Ok(ExprLitBool { value, span }),
            _ => Err(expr),
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
