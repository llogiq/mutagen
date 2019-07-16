//! Mutator for int literals.

use crate::MutagenRuntimeConfig;

pub struct MutatorLitInt<T> {
    mutator_id: u32,
    original_lit: T,
}

// trait for operations that mutate integers of any type
pub trait IntMutable {
    fn add_one(self) -> Self;
}

impl<T: IntMutable> MutatorLitInt<T> {
    pub fn new(mutator_id: u32, original_lit: T) -> Self {
        Self {
            mutator_id,
            original_lit,
        }
    }

    pub fn run_mutator(self, runtime: MutagenRuntimeConfig) -> T {
        if runtime.mutation_id != self.mutator_id {
            self.original_lit
        } else {
            self.original_lit.add_one()
        }
    }
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
        let mutator = MutatorLitInt::new(1, 0);
        let result = mutator.run_mutator(MutagenRuntimeConfig::with_mutation_id(0));
        assert_eq!(result, 0)
    }

    #[test]
    pub fn mutator_lit_int_zero_active() {
        let mutator = MutatorLitInt::new(1, 0);
        let result = mutator.run_mutator(MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, 1)
    }

    #[test]
    fn lit_u8_suffixed() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            let mutator = MutatorLitInt::new(1u32, 1u8);
            let result = mutator.run_mutator(MutagenRuntimeConfig::get_default());
            assert_eq!(result, 2);
        })
    }

    mod test_sum_u32 {
        use crate::mutate;
        use ::mutagen::MutagenRuntimeConfig;

        // test that literals, that are nested in a outside expressen, are mutated
        #[mutate(conf(local), only(lit_int))]
        fn sum_u32() -> u32 {
            1 + 2
        }
        #[test]
        fn sum_u32_inactive() {
            MutagenRuntimeConfig::test_with_mutation_id(0, || {
                assert_eq!(sum_u32(), 3);
            })
        }
        #[test]
        fn sum_u32_active1() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                assert_eq!(sum_u32(), 4);
            })
        }

        #[test]
        fn sum_u32_active2() {
            MutagenRuntimeConfig::test_with_mutation_id(2, || {
                assert_eq!(sum_u32(), 4);
            })
        }
    }

    mod test_lit_u8_suffixed {
        use crate::mutate;
        use ::mutagen::MutagenRuntimeConfig;

        #[mutate(conf(local), only(lit_int))]
        fn lit_u8_suffixed() -> u8 {
            1u8
        }
        #[test]
        fn lit_u8_suffixed_inactive() {
            MutagenRuntimeConfig::test_with_mutation_id(0, || {
                assert_eq!(lit_u8_suffixed(), 1);
            })
        }
        #[test]
        fn lit_u8_suffixed_active() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                assert_eq!(lit_u8_suffixed(), 2);
            })
        }
    }
}
