//! Mutator for int literals.

use syn::{parse_quote, Expr, ExprLit, Lit};

use crate::transform_info::SharedTransformInfo;
use crate::transformer::ExprTransformerOutput;
use crate::Mutation;

use crate::MutagenRuntimeConfig;

pub struct MutatorLitInt {}

impl MutatorLitInt {
    pub fn run<T: IntMutable>(
        mutator_id: u32,
        original_lit: T,
        runtime: MutagenRuntimeConfig,
    ) -> T {
        if runtime.mutation_id != mutator_id {
            original_lit
        } else {
            original_lit.add_one()
        }
    }

    pub fn transform(e: Expr, transform_info: &SharedTransformInfo) -> ExprTransformerOutput {
        match e {
            Expr::Lit(ExprLit {
                lit: Lit::Int(l), ..
            }) => {
                let mutator_id = transform_info
                    .add_mutation(Mutation::new_spanned("lit_int".to_owned(), l.span()));
                let expr = parse_quote! {
                    ::mutagen::mutator::MutatorLitInt::run(
                            #mutator_id,
                            #l,
                            ::mutagen::MutagenRuntimeConfig::get_default()
                        )
                };
                ExprTransformerOutput::changed(expr, l.span())
            }
            _ => ExprTransformerOutput::unchanged(e),
        }
    }
}

// trait for operations that mutate integers of any type
pub trait IntMutable {
    fn add_one(self) -> Self;
}

// implementation for `IntMutable` for all integer types
macro_rules! lit_int_mutables {
    { $($suf:ident, $ty:ident),* } => {
        $(
            impl IntMutable for $ty {
                fn add_one(self) -> Self {
                    self.checked_add(1).expect("overflow")
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

    #[test]
    pub fn mutator_lit_int_zero_inactive() {
        let result = MutatorLitInt::run(1, 0, MutagenRuntimeConfig::with_mutation_id(0));
        assert_eq!(result, 0)
    }

    #[test]
    pub fn mutator_lit_int_zero_active() {
        let result = MutatorLitInt::run(1, 0, MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, 1)
    }

    #[test]
    fn lit_u8_suffixed() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            let result = MutatorLitInt::run(1u32, 1u8, MutagenRuntimeConfig::get_default());
            assert_eq!(result, 2);
        })
    }

}
