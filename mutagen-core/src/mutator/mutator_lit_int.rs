//! Mutator for int literals.

use std::ops::Deref;

use proc_macro2::Span;
use quote::quote_spanned;
use syn::{Expr, ExprLit, Lit};

use crate::transformer::transform_info::SharedTransformInfo;
use crate::Mutation;

use crate::MutagenRuntimeConfig;

pub struct MutatorLitInt {}

impl MutatorLitInt {
    pub fn run<T: IntMutable>(
        mutator_id: u32,
        original_lit: T,
        runtime: impl Deref<Target = MutagenRuntimeConfig>,
    ) -> T {
        runtime.covered(mutator_id);
        let mutations = MutationLitInt::possible_mutations(original_lit.as_u64());
        if let Some(m) = runtime.get_mutation(mutator_id, &mutations) {
            m.mutate(original_lit)
        } else {
            original_lit
        }
    }

    pub fn transform(e: Expr, transform_info: &SharedTransformInfo) -> Expr {
        match e {
            Expr::Lit(ExprLit {
                lit: Lit::Int(lit),
                attrs,
            }) => {
                let lit_val = match lit.base10_parse::<u64>() {
                    Ok(v) => v,
                    Err(_) => {
                        return Expr::Lit(ExprLit {
                            lit: Lit::Int(lit),
                            attrs,
                        })
                    }
                };
                let mutator_id = transform_info.add_mutations(
                    MutationLitInt::possible_mutations(lit_val)
                        .into_iter()
                        .map(|m| m.to_mutation(lit_val, lit.span())),
                );
                syn::parse2(quote_spanned! {lit.span()=>
                    ::mutagen::mutator::MutatorLitInt::run(
                            #mutator_id,
                            #lit,
                            ::mutagen::MutagenRuntimeConfig::get_default()
                        )
                })
                .expect("transformed code invalid")
            }
            _ => e,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum MutationLitInt {
    Relative(i64),
}

impl MutationLitInt {
    fn possible_mutations(val: u64) -> Vec<Self> {
        let mut mutations = vec![];
        if val != u64::max_value() {
            mutations.push(MutationLitInt::Relative(1));
        }
        if val != 0 {
            mutations.push(MutationLitInt::Relative(-1));
        }
        mutations
    }

    fn mutate<T: IntMutable>(self, val: T) -> T {
        match self {
            Self::Relative(r) => {
                IntMutable::from_u64((i128::from(val.as_u64()) + i128::from(r)) as u64)
            }
        }
    }

    fn to_mutation(self, val: u64, span: Span) -> Mutation {
        Mutation::new_spanned(
            "lit_int".to_owned(),
            format!("{}", val),
            format!("{}", self.mutate::<u64>(val)),
            span,
        )
    }
}

// trait for operations that mutate integers of any type
pub trait IntMutable: Copy {
    fn from_u64(val: u64) -> Self;
    fn as_u64(self) -> u64;
}

// implementation for `IntMutable` for all integer types
macro_rules! lit_int_mutables {
    { $($suf:ident, $ty:ident),* } => {
        $(
            impl IntMutable for $ty {
                fn from_u64(val: u64) -> Self {
                    val as $ty
                }
                fn as_u64(self) -> u64 {
                    self as u64
                }
            }
        )*

    }
}

lit_int_mutables! {
    I8, i8,
    I16, i16,
    I32, i32,
    I64, i64,
    I128, i128,
    Isize, isize,
    U8, u8,
    U16, u16,
    U32, u32,
    U64, u64,
    U128, u128,
    Usize, usize
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::MutagenRuntimeConfig;

    #[test]
    pub fn mutator_lit_int_zero_inactive() {
        let result = MutatorLitInt::run(1, 0, &MutagenRuntimeConfig::without_mutation());
        assert_eq!(result, 0)
    }

    #[test]
    pub fn mutator_lit_int_zero_active() {
        let result = MutatorLitInt::run(1, 0, &MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, 1)
    }

    #[test]
    fn lit_u8_suffixed_active() {
        let result: u8 = MutatorLitInt::run(1u32, 1u8, &MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, 2);
    }

    #[test]
    fn possible_mutations_with_zero() {
        assert_eq!(
            MutationLitInt::possible_mutations(0),
            vec![MutationLitInt::Relative(1)]
        );
    }

    #[test]
    fn possible_mutations_with_one() {
        assert_eq!(
            MutationLitInt::possible_mutations(1),
            vec![MutationLitInt::Relative(1), MutationLitInt::Relative(-1)]
        );
    }

    #[test]
    fn possible_mutations_with_max_value() {
        assert_eq!(
            MutationLitInt::possible_mutations(u64::max_value()),
            vec![MutationLitInt::Relative(-1)]
        );
    }

    #[test]
    fn mutate_relative1() {
        assert_eq!(MutationLitInt::Relative(1).mutate(2), 3)
    }

    #[test]
    fn mutate_relative_neg1() {
        assert_eq!(MutationLitInt::Relative(-1).mutate(2), 1)
    }
}
