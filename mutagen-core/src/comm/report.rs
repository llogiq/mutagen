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
        assert!(!self.mutant_results.contains_key(&mutation));
        // TODO: instead use: .expect_none("mutation already added");
        self.mutant_results.insert(mutation, status);
        self.summary.add_mutation_result(status);
    }

    pub fn print_survived(&self) {
        if self.summary.survived > 0 {
            println!("SURVIVED");
            let survived = group(
                self.mutant_results
                    .iter()
                    .filter(|(_, s)| s.survived())
                    .map(|(m, s)| (m.source_file(), (m, *s))),
            );

            for (file, mutations) in survived {
                println!("    {}", file.display());
                for (m, s) in mutations {
                    println!(
                        "        {}: {} at {}{}{}",
                        m.id(),

                        m.mutation_description(),
                        m.location_in_file(),
                        m.context_description_in_brackets(),
                        if s == MutantStatus::NotCovered {
                            format!(" {}", MutantStatus::NotCovered)
                        } else {
                            "".to_owned()
                        },
                    );
                }
            }
        }
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

fn group<K: Eq + std::hash::Hash, V, I: Iterator<Item = (K, V)>>(iter: I) -> HashMap<K, Vec<V>> {
    iter.fold(HashMap::new(), |mut map, (k, v)| {
        map.entry(k).or_insert_with(|| Vec::new()).push(v);
        map
    })
}

/// Result from a test run
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
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

impl MutantStatus {
    fn survived(self) -> bool {
        self == Self::NotCovered || self == Self::Survived
    }
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
