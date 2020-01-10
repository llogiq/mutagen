//! Tells mutators what to do at runtime
//!
//! Currently, 3 modes are supported
//!
//! * do nothing
//! * activate a single mutation
//! * report the coverage of mutators
//!
//! The main method to configure the global runtime configuration is via environment variables.
//! The variable `MUTATION_ID` activates a single mutation
//! The variable `MUTAGEN_MODE` is used to specify other configurations.
//!
//! * `MUTAGEN_MODE=mutation`: activate a single mutation (default)
//! * `MUTAGEN_MODE=coverage`: perform coverage analysis
//!
//! In the mode `coverage`, it is required to add the environment variable `MUTAGEN_NUM_MUTATIONS=N` where `N` are the total number of mutations

use lazy_static::lazy_static;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

use crate::comm;

lazy_static! {
    static ref RUNTIME_CONFIG: RwLock<MutagenRuntimeConfig> =
        {
            // sets the global config such that
            // * config constructed via `from_env` when outside tests
            // * during tests, `from_env` is not called
            #[cfg(not(any(test, feature = "self_test")))]
            let config = MutagenRuntimeConfig::from_env();
            #[cfg(any(test, feature = "self_test"))]
            let config = MutagenRuntimeConfig::without_mutation();
            RwLock::new(config)
        };
}

pub enum MutagenRuntimeConfig {
    Pass,
    Mutation(usize),
    Coverage(CoverageRecorder),
}

/// Counts how many times each mutator has been covered and reports when a mutator is covered the first time.
pub struct CoverageRecorder {
    coverage: CoverageHitCollector,
    coverage_file: File,
}

impl MutagenRuntimeConfig {
    /// Sccess the currently active runtime-config based on the environment variable `MUATION_ID`.
    ///
    /// During tests, the global runtime_config can be set to any value to allow
    /// exhaustive testing.
    pub fn get_default() -> impl Deref<Target = Self> {
        RUNTIME_CONFIG.read().unwrap()
    }

    /// Creates a runtime config from environment variables.
    ///
    /// See the module documentation for configuration options
    #[cfg_attr(any(test, feature = "self_test"), allow(dead_code))]
    // private fn `from_env` is not used when during test (cfg-switch in RUNTIME_CONFIG)
    fn from_env() -> Self {
        let mode = std::env::var("MUTAGEN_MODE").ok().unwrap_or("".to_owned());
        match &*mode {
            "coverage" => {
                let num_mutations = std::env::var("MUTAGEN_NUM_MUTATIONS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .expect("environemnt variable `MUTAGEN_NUM_MUTATIONS` missing");
                Self::Coverage(CoverageRecorder::new(num_mutations))
            }
            "" | "mutation" => {
                let mutation_id = std::env::var("MUTATION_ID")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                if mutation_id == 0 {
                    Self::Pass
                } else {
                    Self::Mutation(mutation_id)
                }
            }
            m => panic!("unknown mutagen mode: `{}`", m),
        }
    }

    /// Records that mutator with the given id is covered.
    ///
    /// This does nothing if coverage is not enabled.
    pub fn covered(&self, mutator_id: usize) {
        if let Self::Coverage(coverage) = &self {
            coverage.covered(mutator_id)
        }
    }

    /// Function to abort the computation in case a optimistic mutation fails.
    ///
    /// In the future, this will be configurable
    pub fn optimistic_assmuption_failed(&self) -> ! {
        match self {
            Self::Mutation(m_id) => {
                panic!("optimistic assumption failed for mutation {}", m_id);
            }
            _ => panic!("optimistic assumption failed without mutation"),
        }
    }

    pub fn mutation_id(&self) -> Option<usize> {
        if let Self::Mutation(m_id) = self {
            Some(*m_id)
        } else {
            None
        }
    }

    /// Checks if the given mutation is activated.
    pub fn is_mutation_active(&self, mutation_id: usize) -> bool {
        self.mutation_id() == Some(mutation_id)
    }

    /// Returns the active mutation for a given mutator, or None if no mutation of the mutator is activated.
    pub fn get_mutation_for_mutator<'a, T>(
        &self,
        mutator_id: usize,
        mutations: &'a [T],
    ) -> Option<&'a T> {
        let m_id = self.mutation_id()?;
        if m_id < mutator_id {
            return None;
        }
        let index = m_id - mutator_id;
        mutations.get(index)
    }
}

impl CoverageRecorder {
    fn new(num_mutations: usize) -> Self {
        let coverage = CoverageHitCollector::new(num_mutations);
        let coverage_filepath = comm::get_coverage_file().unwrap();
        let coverage_file = File::create(&coverage_filepath)
            .unwrap_or_else(|_| panic!("unable to open file {:?}", &coverage_filepath));

        Self {
            coverage,
            coverage_file,
        }
    }

    fn covered(&self, mutator_id: usize) {
        // report first coverage
        if self.coverage.hit(mutator_id) {
            let coverage_hit = comm::CoverageHit { mutator_id };

            let mut w = BufWriter::new(&self.coverage_file);
            serde_json::to_writer(&mut w, &coverage_hit).expect("unable to write to coverage file");
            // write newline
            writeln!(&mut w).expect("unable to write to coverage file");
        }
    }
}

/// struct that collects coverage of mutators.
///
/// It has to be created with a known size.
///
/// The method `hit`, is used for recording coverage hits.
struct CoverageHitCollector(Vec<AtomicU64>);

impl CoverageHitCollector {
    /// constructs a HotCoverageCollection for a given number of mutations
    fn new(num_mutations: usize) -> Self {
        Self((0..=num_mutations).map(|_| AtomicU64::new(0)).collect())
    }

    /// records a single coverage hit.
    ///
    /// Returns true iff this hit was the first for this mutator
    fn hit(&self, mutator_id: usize) -> bool {
        0 == self.0[mutator_id].fetch_add(1, Ordering::Relaxed)
    }
}

/// module with functions used for isolated and exhaustive tests of the `#[mutate]` attribute
#[cfg(any(test, feature = "self_test"))]
mod test_tools {

    use super::*;
    use std::sync::Mutex;

    lazy_static! {
        /// a lock to ensure that the tests are run sequentially since global information is set.
        static ref TEST_LOCK: Mutex<()> = Mutex::new(());
    }

    impl MutagenRuntimeConfig {
        /// sets the global `mutation_id` correctly before running the test and runs tests sequentially.
        ///
        /// The lock is required to ensure that set `mutation_id` is valid for the complete duration of the test case.
        fn test_with_runtime<F: FnOnce() -> ()>(self, testcase: F) {
            let lock = TEST_LOCK.lock();
            *RUNTIME_CONFIG.write().unwrap() = self;
            testcase();
            drop(lock); // drop here to show the extended lifetime of lock guard
        }

        pub fn test_without_mutation<F: FnOnce() -> ()>(testcase: F) {
            Self::test_with_runtime(Self::without_mutation(), testcase)
        }

        pub fn test_with_mutation_id<F: FnOnce() -> ()>(mutation_id: usize, testcase: F) {
            Self::test_with_runtime(Self::with_mutation_id(mutation_id), testcase)
        }

        pub fn without_mutation() -> Self {
            Self::Pass
        }

        pub fn with_mutation_id(mutation_id: usize) -> Self {
            assert!(mutation_id != 0);
            MutagenRuntimeConfig::Mutation(mutation_id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_mutation_active() {
        let config = MutagenRuntimeConfig::with_mutation_id(1);

        assert!(config.is_mutation_active(1));
    }
    #[test]
    fn config_mutation_inactive() {
        let config = MutagenRuntimeConfig::with_mutation_id(1);

        assert!(!config.is_mutation_active(2));
    }
    #[test]
    fn config_mutation_no_mutation() {
        let config = MutagenRuntimeConfig::without_mutation();

        assert!(!config.is_mutation_active(1));
    }

    #[test]
    fn coverage_hit_collector_hit() {
        let collector = CoverageHitCollector::new(1);
        assert!(collector.hit(1));
    }
    #[test]
    fn coverage_hit_collector_repeated_hit() {
        let collector = CoverageHitCollector::new(1);
        collector.hit(1);

        assert!(!collector.hit(1));
    }
    #[test]
    fn coverage_hit_collector_hit_different_mutators() {
        let collector = CoverageHitCollector::new(2);
        assert!(collector.hit(1));
        assert!(collector.hit(2));
    }
    #[test]
    #[should_panic]
    fn coverage_hit_collector_out_of_bounds() {
        CoverageHitCollector::new(1).hit(2);
    }
}
