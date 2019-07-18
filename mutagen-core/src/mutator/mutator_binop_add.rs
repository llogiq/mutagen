//! Mutator for binary operation `+`.

use std::ops::Add;

use crate::optimistic::AddToSub;
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

}
