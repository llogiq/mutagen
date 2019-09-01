use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt;

use super::BakedMutation;

#[derive(Serialize, Deserialize, Default)]
pub struct MutagenReport {
    mutant_results: HashMap<BakedMutation, MutantStatus>,
    summary: ReportSummary,
}

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub struct ReportSummary {
    num_mutations: usize,
    killed: usize,
    timeout: usize,
    survived: usize,
    not_covered: usize,
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

    /// creates a map of mutations per file.
    ///
    /// The map gets iterated in alphabetical order of the files and the list of mutations is sorted by mutation-id
    fn mutations_per_file(
        &self,
    ) -> BTreeMap<&std::path::Path, Vec<(&BakedMutation, MutantStatus)>> {
        let mut map = BTreeMap::new();
        // collect mutations by source file
        for (m, s) in &self.mutant_results {
            map.entry(m.source_file()).or_insert(vec![]).push((m, *s));
        }
        // sort list of mutations per source file by id
        for (_, ms) in &mut map {
            ms.sort_unstable_by_key(|(m, _)| m.id());
        }
        map
    }

    pub fn print_survived(&self) {
        println!("SURVIVED");
        let mutations_per_file = self.mutations_per_file().into_iter().collect::<Vec<_>>();

        for (file, mutations) in mutations_per_file {
            let num_mutations = mutations.len();
            // TODO: use mutations.drain_filter
            let survived = mutations
                .into_iter()
                .filter(|(_, s)| s.survived())
                .collect::<Vec<_>>();
            let num_survived = survived.len();

            println!("    {}", file.display());
            if num_survived == 0 {
                println!("            all {} mutants killed", num_mutations);
            } else if num_survived == num_mutations {
                println!("            all {} mutants survived", num_survived);
            } else {
                println!(
                    "            {}/{}({:.2}%) mutants survived",
                    num_survived,
                    num_mutations,
                    compute_percent(num_mutations, num_survived)
                );
            }
            for (m, s) in survived {
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
        let percent_mutations_killed = compute_percent(self.num_mutations, self.killed);
        let percent_mutations_timeout = compute_percent(self.num_mutations, self.timeout);
        let percent_mutations_survived = compute_percent(self.num_mutations, self.survived);
        let percent_mutations_not_covered = compute_percent(self.num_mutations, self.not_covered);

        println!();
        println!("{} generated mutations", self.num_mutations);
        println!(
            "{}({:.2}%) mutants killed, {}({:.2}%) by timeout",
            self.killed, percent_mutations_killed, self.timeout, percent_mutations_timeout,
        );
        println!(
            "{}({:.2}%) mutants SURVIVED, {}({:.2}%) NOT COVERED",
            self.survived,
            percent_mutations_survived,
            self.not_covered,
            percent_mutations_not_covered,
        );
    }
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

fn compute_percent(total: usize, num: usize) -> f64 {
    100.0 * num as f64 / total as f64
}
