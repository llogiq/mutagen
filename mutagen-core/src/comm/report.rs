use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use super::BakedMutation;

#[derive(Serialize, Deserialize, Default)]
pub struct MutagenReport {
    mutant_results: HashMap<BakedMutation, MutantStatus>,
    summary: ReportSummary,
}

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub struct ReportSummary {
    num_mutations: u32,
    killed: u32,
    timeout: u32,
    survived: u32,
    not_covered: u32,
}

impl MutagenReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_mutation_result(&mut self, mutation: BakedMutation, status: MutantStatus) {
        assert!(!self.mutant_results.contains_key(&mutation)); // TODO: use this assert?
        self.mutant_results.insert(mutation, status);
        self.summary.add_mutation_result(status);
    }

    pub fn summary(&self) -> ReportSummary {
        self.summary
    }
}

impl ReportSummary {
    fn add_mutation_result(&mut self, status: MutantStatus) {
        self.num_mutations += 1;
        match status {
            MutantStatus::NotCovered => {
                self.not_covered += 1;
                self.survived += 1;
            }
            MutantStatus::Survived => self.survived += 1,
            MutantStatus::Killed(_) => self.killed += 1,
            MutantStatus::Timeout => {
                self.timeout += 1;
                self.killed += 1;
            }
        }
    }

    pub fn print(&self) {
        let coverage = 100.0 * self.killed as f64 / self.num_mutations as f64;

        println!();
        println!(
            "{} mutants killed ({} by timeout)",
            self.killed, self.timeout
        );
        println!(
            "{} mutants SURVIVED ({} NOT COVERED)",
            self.survived, self.not_covered
        );
        println!("{:.2}% mutation coverage", coverage);
    }
}

/// Result from a test run
#[derive(PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum MutantStatus {
    /// The test suite did not cover the mutator
    NotCovered,
    /// test pass
    Survived,
    /// the test broke with an error code
    Killed(Option<i32>),
    /// the test timed out
    Timeout,
}

impl fmt::Display for MutantStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotCovered => write!(f, "NOT COVERED"),
            Self::Survived => write!(f, "SURVIVED"),
            Self::Killed(_) => write!(f, "killed"),
            Self::Timeout => write!(f, "killed (timeout)"),
        }
    }
}
