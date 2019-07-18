//! Mutator for binary operation `+`.

use std::ops::Not;

use crate::optimistic::NotToNone;
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

    pub use crate::optimistic::{TypeWithNotOtherOutput, TypeWithNotTarget};

    #[test]
    fn optimistic_incorrect_inactive() {
        let mutator = MutatorUnopNot::new(1, TypeWithNotOtherOutput());
        let result = mutator.run_mutator(MutagenRuntimeConfig::with_mutation_id(0));
        assert_eq!(result, TypeWithNotTarget());
    }
    #[test]
    #[should_panic]
    fn optimistic_incorrect_active() {
        let mutator = MutatorUnopNot::new(1, TypeWithNotOtherOutput());
        mutator.run_mutator(MutagenRuntimeConfig::with_mutation_id(1));
    }

}
