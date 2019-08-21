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
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

use crate::mutagen_file::get_coverage_file;

lazy_static! {
    static ref RUNTIME_CONFIG: RwLock<MutagenRuntimeConfig> =
        {
            // sets the global config such that
            // * during tests, `from_env` is not called
            // * config constructed via `from_env` when outside tests
            #[cfg(not(any(test, feature = "self_test")))]
            let config = MutagenRuntimeConfig::from_env();
            #[cfg(any(test, feature = "self_test"))]
            let config = MutagenRuntimeConfig::without_mutation();
            RwLock::new(config)
        };
}

pub enum MutagenRuntimeConfig {
    Pass,
    Mutation(u32),
    Coverage(CoverageCounter),
}

/// counts how many times each mutator has been covered and reports when a mutator is covered the first time
pub struct CoverageCounter {
    counter: Vec<AtomicU64>,
    coverage_file: File,
}

#[derive(Serialize, Deserialize)]
struct CoverageHit {
    mutator_id: u32,
}

impl MutagenRuntimeConfig {
    /// access the currently active runtime-config based on the environment variable `MUATION_ID`.
    ///
    /// during tests, the global runtime_config can be set to any value to allow
    /// exhaustive testing.
    pub fn get_default() -> impl Deref<Target = Self> {
        RUNTIME_CONFIG.read().unwrap()
    }

    #[cfg_attr(any(test, feature = "self_test"), allow(dead_code))]
    fn from_env() -> Self {
        let mode = std::env::var("MUTAGEN_MODE").ok().unwrap_or("".to_owned());
        match &*mode {
            "coverage" => {
                let num_mutations = std::env::var("MUTAGEN_NUM_MUTATIONS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .expect("environemnt variable `MUTAGEN_NUM_MUTATIONS` missing");
                Self::Coverage(CoverageCounter::new(num_mutations))
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

    pub fn covered(&self, mutation_id: u32) {
        if let Self::Coverage(coverage) = &self {
            coverage.covered(mutation_id)
        }
    }

    pub fn mutation_id(&self) -> Option<u32> {
        if let Self::Mutation(m_id) = self {
            Some(*m_id)
        } else {
            None
        }
    }

    pub fn is_mutation_active(&self, mutation_id: u32) -> bool {
        self.mutation_id() == Some(mutation_id)
    }

    pub fn get_mutation<'a, T>(&self, mutator_id: u32, mutations: &'a [T]) -> Option<&'a T> {
        let m_id = self.mutation_id()?;
        if m_id < mutator_id {
            return None;
        }
        let index = m_id - mutator_id;
        mutations.get(index as usize)
    }
}

impl CoverageCounter {
    fn new(max_mutations: u32) -> Self {
        let counter = (0..=max_mutations).map(|_| AtomicU64::new(0)).collect();
        let coverage_filepath = get_coverage_file().unwrap();
        let coverage_file = File::create(&coverage_filepath)
            .unwrap_or_else(|_| panic!("unable to open file {:?}", &coverage_filepath));

        Self {
            counter,
            coverage_file,
        }
    }

    fn covered(&self, mutator_id: u32) {
        let previous_cover_counter =
            self.counter[mutator_id as usize].fetch_add(1, Ordering::Relaxed);
        // report first coverage
        if previous_cover_counter == 0 {
            let coverage_hit = CoverageHit { mutator_id };

            let mut w = BufWriter::new(&self.coverage_file);
            serde_json::to_writer(&mut w, &coverage_hit).expect("unable to write to coverage file");
            // write newline
            writeln!(&mut w).expect("unable to write to coverage file");
        }
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

        pub fn test_with_mutation_id<F: FnOnce() -> ()>(mutation_id: u32, testcase: F) {
            Self::test_with_runtime(Self::with_mutation_id(mutation_id), testcase)
        }

        pub fn without_mutation() -> Self {
            Self::Pass
        }

        pub fn with_mutation_id(mutation_id: u32) -> Self {
            assert!(mutation_id != 0);
            MutagenRuntimeConfig::Mutation(mutation_id)
        }
    }
}
