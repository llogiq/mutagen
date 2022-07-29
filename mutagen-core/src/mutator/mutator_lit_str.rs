//! Mutator for str literals.

use std::convert::TryFrom;
use std::ops::Deref;

use proc_macro2::Span;
use quote::quote_spanned;
use syn::{Expr, Lit, LitStr};

use crate::comm::Mutation;
use crate::transformer::transform_info::SharedTransformInfo;
use crate::transformer::TransformContext;

use crate::MutagenRuntimeConfig;

pub fn run(
    mutator_id: usize,
    original_lit: &'static str,
    mutations: &[&'static str],
    runtime: &impl Deref<Target = MutagenRuntimeConfig>,
) -> &'static str {
    runtime.covered(mutator_id);
    if let Some(m) = runtime.get_mutation_for_mutator(mutator_id, &mutations) {
        m
    } else {
        original_lit
    }
}

pub fn transform(
    e: Expr,
    transform_info: &SharedTransformInfo,
    context: &TransformContext,
) -> Expr {
    let e = match ExprLitStr::try_from(e) {
        Ok(e) => e,
        Err(e) => return e,
    };

    let possible_mutations = MutationLitStr::possible_mutations(e.clone().value);
    let mutations: Vec<_> = possible_mutations
        .iter()
        .map(|x| x.mutate(&e.clone().value))
        .collect();

    let mutator_id = transform_info.add_mutations(
        possible_mutations
            .into_iter()
            .map(|m| m.to_mutation(&e, context)),
    );

    let original_lit = e.lit.value();

    syn::parse2(quote_spanned! {e.span=>
        ::mutagen::mutator::mutator_lit_str::run(
                #mutator_id,
                #original_lit,
                &[#(&#mutations),*], // Expands to `&[mutations[0], mutations[1], ..., [mutations[n]]]`
                &::mutagen::MutagenRuntimeConfig::get_default()
            )
    })
    .expect("transformed code invalid")
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum MutationLitStr {
    Clear,
    Set(&'static str),
    Append(char),
    Prepend(char),
}

impl MutationLitStr {
    fn possible_mutations(val: String) -> Vec<Self> {
        let mut mutations = vec![];
        if val.is_empty() {
            mutations.push(Self::Set("A"))
        } else {
            mutations.push(Self::Clear);
            mutations.push(Self::Prepend('-'));
            mutations.push(Self::Append('-'));
        }
        mutations
    }

    fn mutate(&self, val: &str) -> String {
        match self {
            Self::Clear => "".to_string(),
            Self::Set(string) => string.to_string(),
            Self::Append(char) => {
                let mut new = val.to_string();
                new.push(*char);
                new
            }
            Self::Prepend(char) => {
                let mut new = val.to_string();
                new.insert(0, *char);
                new
            }
        }
    }

    fn to_mutation(&self, original_lit: &ExprLitStr, context: &TransformContext) -> Mutation {
        Mutation::new_spanned(
            context,
            "lit_str".to_owned(),
            original_lit.value.to_string(),
            self.mutate(&original_lit.value).to_string(),
            original_lit.span,
        )
    }
}

#[derive(Clone, Debug)]
pub struct ExprLitStr {
    pub value: String,
    pub lit: LitStr,
    pub span: Span,
}

impl TryFrom<Expr> for ExprLitStr {
    type Error = Expr;
    fn try_from(expr: Expr) -> Result<Self, Expr> {
        match expr {
            Expr::Lit(expr) => match expr.lit {
                Lit::Str(lit) => Ok(ExprLitStr {
                    value: lit.value(),
                    span: lit.span(),
                    lit,
                }),
                _ => Err(Expr::Lit(expr)),
            },
            _ => Err(expr),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::MutagenRuntimeConfig;

    #[test]
    pub fn mutator_lit_str_empty_inactive() {
        let result = run(1, "", &["A"], &&MutagenRuntimeConfig::without_mutation());
        assert_eq!(result, "")
    }

    #[test]
    pub fn mutator_lit_str_non_empty_inactive() {
        let result = run(
            1,
            "ABCD",
            &["", "-ABCD", "ABCD-"],
            &&MutagenRuntimeConfig::without_mutation(),
        );
        assert_eq!(result, "ABCD")
    }

    #[test]
    pub fn mutator_lit_str_non_empty_active_1() {
        let result = run(
            1,
            "a",
            &["", "-ABCD", "ABCD-"],
            &&MutagenRuntimeConfig::with_mutation_id(1),
        );
        assert_eq!(result, "")
    }

    #[test]
    pub fn mutator_lit_str_non_empty_active_2() {
        let result = run(
            1,
            "a",
            &["", "-ABCD", "ABCD-"],
            &&MutagenRuntimeConfig::with_mutation_id(2),
        );
        assert_eq!(result, "-ABCD")
    }

    #[test]
    pub fn mutator_lit_str_non_empty_active_3() {
        let result = run(
            1,
            "a",
            &["", "-ABCD", "ABCD-"],
            &&MutagenRuntimeConfig::with_mutation_id(3),
        );
        assert_eq!(result, "ABCD-")
    }

    #[test]
    pub fn mutator_lit_str_empty_active_1() {
        let result = run(1, "", &["A"], &&MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, "A")
    }
}
