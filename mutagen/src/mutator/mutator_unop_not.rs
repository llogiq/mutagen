//! Mutator for binary operation `+`.

use std::ops::Not;

use crate::optimistic::not_to_none::NotToNone;
use crate::MutagenRuntimeConfig;

pub struct MutatorUnopNot<T> {
    mutator_id: u32,
    val: T,
}

impl<T: Not> MutatorUnopNot<T> {
    pub fn new(mutator_id: u32, val: T) -> Self {
        Self { mutator_id, val }
    }

    pub fn run_mutator(self, runtime: MutagenRuntimeConfig) -> <T as Not>::Output {
        if runtime.mutation_id != self.mutator_id {
            !self.val
        } else {
            self.val.may_none()
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn boolnot_inactive() {
        // input is true, but will be negated by non-active mutator
        let mutator = MutatorUnopNot::new(1, true);
        let result = mutator.run_mutator(MutagenRuntimeConfig::with_mutation_id(0));
        assert_eq!(result, false);
    }
    #[test]
    fn boolnot_active() {
        let mutator = MutatorUnopNot::new(1, true);
        let result = mutator.run_mutator(MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, true);
    }
    #[test]
    fn intnot_active() {
        let mutator = MutatorUnopNot::new(1, 1);
        let result = mutator.run_mutator(MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, 1);
    }

    // types for test cases where optimistic assumption fails
    #[derive(Debug, PartialEq)]
    struct OptimisticTestTypeX();
    #[derive(Debug, PartialEq)]
    struct OptimisticTestTypeY();

    impl Not for OptimisticTestTypeX {
        type Output = OptimisticTestTypeY;

        fn not(self) -> <Self as Not>::Output {
            OptimisticTestTypeY()
        }
    }

    #[test]
    fn optimistic_incorrect_inactive() {
        let mutator = MutatorUnopNot::new(1, OptimisticTestTypeX());
        let result = mutator.run_mutator(MutagenRuntimeConfig::with_mutation_id(0));
        assert_eq!(result, OptimisticTestTypeY());
    }
    #[test]
    #[should_panic]
    fn optimistic_incorrect_active() {
        let mutator = MutatorUnopNot::new(1, OptimisticTestTypeX());
        mutator.run_mutator(MutagenRuntimeConfig::with_mutation_id(1));
    }

    mod test_boolnot {

        use crate::mutate;
        use ::mutagen::MutagenRuntimeConfig;

        // simple test that sums 2 u32 values. Unfortunately, the tag `u32` is necessary
        #[mutate(conf(local), only(unop_not))]
        fn boolnot(x: bool) -> bool {
            !x
        }
        #[test]
        fn boolnot_false_inactive() {
            MutagenRuntimeConfig::test_with_mutation_id(0, || {
                assert_eq!(boolnot(false), true);
            })
        }
        #[test]
        fn boolnot_false_active() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                assert_eq!(boolnot(false), false);
            })
        }
        #[test]
        fn boolnot_true_active() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                assert_eq!(boolnot(true), true);
            })
        }
    }

    mod test_optimistic_incorrect {

        use super::*;
        use crate::mutate;
        use ::mutagen::MutagenRuntimeConfig;

        // strings cannot be subtracted, the mutation that changes `+` into `-` should panic
        #[mutate(conf(local), only(unop_not))]
        fn optimistic_incorrect(x: OptimisticTestTypeX) -> OptimisticTestTypeY {
            !x
        }
        #[test]
        fn optimistic_incorrect_inactive() {
            MutagenRuntimeConfig::test_with_mutation_id(0, || {
                assert_eq!(
                    optimistic_incorrect(OptimisticTestTypeX()),
                    OptimisticTestTypeY()
                );
            })
        }
        #[test]
        #[should_panic]
        fn optimistic_incorrect_active1() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                optimistic_incorrect(OptimisticTestTypeX());
            })
        }
    }
}
