//! Mutator for binary operation `+`.

use std::ops::Add;

use crate::optimistic::add_to_sub::AddToSub;
use crate::MutagenRuntimeConfig;

pub struct MutatorBinopAdd<L, R> {
    mutator_id: u32,
    left: L,
    right: R,
}

impl<L: Add<R>, R> MutatorBinopAdd<L, R> {
    pub fn new(mutator_id: u32, left: L, right: R) -> Self {
        Self {
            mutator_id,
            left,
            right,
        }
    }

    pub fn run_mutator(self, runtime: MutagenRuntimeConfig) -> <L as Add<R>>::Output {
        if runtime.mutation_id != self.mutator_id {
            self.left + self.right
        } else {
            self.left.may_sub(self.right)
        }
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn sum_inative() {
        let mutator = MutatorBinopAdd::new(1, 5, 4);
        let result = mutator.run_mutator(MutagenRuntimeConfig::with_mutation_id(0));
        assert_eq!(result, 9);
    }
    #[test]
    fn sum_ative() {
        let mutator = MutatorBinopAdd::new(1, 5, 4);
        let result = mutator.run_mutator(MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, 1);
    }

    #[test]
    fn str_add_inactive() {
        let mutator = MutatorBinopAdd::new(1, "x".to_string(), "y");
            let result = mutator.run_mutator(MutagenRuntimeConfig::with_mutation_id(0));
        assert_eq!(&result, "xy");
    }
    #[test]
    #[should_panic]
    fn str_add_active() {
        let mutator = MutatorBinopAdd::new(1, "x".to_string(), "y");
        mutator.run_mutator(MutagenRuntimeConfig::with_mutation_id(1));
    }

    mod test_sum_u32 {

        use crate::mutate;
        use ::mutagen::MutagenRuntimeConfig;

        // simple test that sums 2 u32 values. Unfortunately, the tag `u32` is necessary
        #[mutate(conf(local), only(binop_add))]
        fn sum_u32() -> u32 {
            5u32 + 1
        }
        #[test]
        fn sum_u32_inactive() {
            MutagenRuntimeConfig::test_with_mutation_id(0, || {
                assert_eq!(sum_u32(), 6);
            })
        }
        #[test]
        fn sum_u32_active1() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                assert_eq!(sum_u32(), 4);
            })
        }

    }

    mod test_str_add {

        use crate::mutate;
        use ::mutagen::MutagenRuntimeConfig;

        // strings cannot be subtracted, the mutation that changes `+` into `-` should panic
        #[mutate(conf(local), only(binop_add))]
        fn str_add() -> String {
            "a".to_string() + "b"
        }
        #[test]
        fn str_add_inactive() {
            MutagenRuntimeConfig::test_with_mutation_id(0, || {
                assert_eq!(&str_add(), "ab");
            })
        }
        #[test]
        #[should_panic]
        fn str_add_active1() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                str_add();
            })
        }
    }

    mod test_multiple_adds {

        use crate::mutate;
        use ::mutagen::MutagenRuntimeConfig;

        // sum of multiple values without brackets
        #[mutate(conf(local), only(binop_add))]
        pub fn multiple_adds(i: usize) -> usize {
            i + 4 + 1
        }

        #[test]
        fn multiple_adds_inactive() {
            MutagenRuntimeConfig::test_with_mutation_id(0, || {
                assert_eq!(multiple_adds(5), 10);
            })
        }
        #[test]
        fn multiple_adds_active1() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                assert_eq!(multiple_adds(5), 2);
            })
        }
        #[test]
        fn multiple_adds_active2() {
            MutagenRuntimeConfig::test_with_mutation_id(2, || {
                assert_eq!(multiple_adds(5), 8);
            })
        }
    }
}
