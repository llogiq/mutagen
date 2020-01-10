use super::BakedMutation;
use serde::{Deserialize, Serialize};

/// A single coverage hit.
#[derive(Debug, Serialize, Deserialize)]
pub struct CoverageHit {
    pub mutator_id: usize,
}

/// A collection that tracks which mutations have been covered.
///
/// The collection can be created my
#[derive(Debug, Serialize, Deserialize)]
pub struct CoverageCollection {
    num_covered: usize,
    coverage: Vec<bool>,
}

impl CoverageCollection {
    /// Create a collection about coverage where no mutator has been covered.
    pub fn new_empty(num_mutations: usize) -> Self {
        Self {
            num_covered: 0,
            coverage: vec![false; num_mutations + 1],
        }
    }

    /// Create a collection about coverage from a list of coverage hits.
    pub fn from_coverage_hits(
        num_mutations: usize,
        hits: &[CoverageHit],
        mutations: &[BakedMutation],
    ) -> Self {
        let mut coverage = vec![false; num_mutations + 1];
        let mut num_covered = 0;

        for hit in hits {
            if !coverage[hit.mutator_id] {
                for m in mutations {
                    if m.mutator_id() == hit.mutator_id {
                        num_covered += 1;
                        coverage[m.id()] = true;
                    }
                }
            }
        }

        Self {
            num_covered,
            coverage,
        }
    }

    /// Merge multiple coverage collections into a single one.
    pub fn merge<'a>(
        num_mutations: usize,
        coverages: impl IntoIterator<Item = &'a CoverageCollection>,
    ) -> Self {
        let mut coverage = vec![false; num_mutations + 1];
        let mut num_covered = 0;

        for c in coverages {
            for m_id in 1..=num_mutations {
                if c.is_covered(m_id) {
                    if !coverage[m_id] {
                        num_covered += 1;
                        coverage[m_id] = true;
                    }
                }
            }
        }

        Self {
            num_covered,
            coverage,
        }
    }

    /// Checks if the given mutation is covered.
    pub fn is_covered(&self, m_id: usize) -> bool {
        self.coverage[m_id]
    }

    /// Returns the number of covered mutations.
    pub fn num_covered(&self) -> usize {
        self.num_covered
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comm::Mutation;

    #[test]
    fn coverage_collection_empty() {
        let c = CoverageCollection::new_empty(2);

        assert!(!c.is_covered(1));
        assert!(!c.is_covered(2))
    }

    #[test]
    fn coverage_collection_sinlge_covered() {
        let c = CoverageCollection::from_coverage_hits(
            2,
            &[CoverageHit { mutator_id: 1 }],
            &[Mutation::new_stub().with_id(1, 1)],
        );

        assert!(c.is_covered(1));
        assert!(!c.is_covered(2));
        assert_eq!(c.num_covered(), 1);
    }
    #[test]
    fn coverage_collection_covered_all_mutations_of_mutator() {
        let c = CoverageCollection::from_coverage_hits(
            2,
            &[CoverageHit { mutator_id: 1 }],
            &[
                Mutation::new_stub().with_id(1, 1),
                Mutation::new_stub().with_id(2, 1),
            ],
        );

        assert!(c.is_covered(1));
        assert!(c.is_covered(2));
        assert_eq!(c.num_covered(), 2);
    }

    #[test]
    fn coverage_collection_merge() {
        let mutations = [
            Mutation::new_stub().with_id(1, 1),
            Mutation::new_stub().with_id(2, 2),
            Mutation::new_stub().with_id(3, 3),
            Mutation::new_stub().with_id(4, 3),
        ];
        let c1 = CoverageCollection::from_coverage_hits(
            4,
            &[CoverageHit { mutator_id: 2 }],
            &mutations,
        );
        let c2 = CoverageCollection::from_coverage_hits(
            4,
            &[CoverageHit { mutator_id: 3 }],
            &mutations,
        );

        let c = CoverageCollection::merge(4, &[c1, c2]);

        assert!(c.is_covered(2));
        assert!(c.is_covered(3));
        assert!(c.is_covered(4));
        assert_eq!(c.num_covered(), 3);
    }
}
