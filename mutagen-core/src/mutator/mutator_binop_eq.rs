//! Mutator for binary operation `==`.

use std::cmp::PartialEq;

use crate::MutagenRuntimeConfig;

pub struct MutatorBinopEq<L, R> {
    mutator_id: u32,
    left: L,
    right: R,
}

impl<L: PartialEq<R>, R> MutatorBinopEq<L, R> {
    pub fn new(mutator_id: u32, left: L, right: R) -> Self {
        Self {
            mutator_id,
            left,
            right,
        }
    }

    pub fn run_mutator(self, runtime: MutagenRuntimeConfig) -> bool {
        if runtime.mutation_id != self.mutator_id {
            self.left == self.right
        } else {
            self.left != self.right
        }
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn eq_inative() {
        let mutator = MutatorBinopEq::new(1, 5, 4);
        let result = mutator.run_mutator(MutagenRuntimeConfig::with_mutation_id(0));
        assert_eq!(result, false);
    }
    #[test]
    fn eq_ative() {
        let mutator = MutatorBinopEq::new(1, 5, 4);
        let result = mutator.run_mutator(MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, true);
    }

}
