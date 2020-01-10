use failure::{bail, Fallible};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use wait_timeout::ChildExt;

use mutagen_core::comm::{self, BakedMutation, CoverageCollection, CoverageHit, MutantStatus};

use super::Progress;

/// wrapper around a test-binary that can be executed
#[derive(Debug)]
pub struct TestBin<'a> {
    id: usize,
    pub bin_path: &'a Path,
}

// wrapper around a test-binary, which has been run already and its runtime has been timed.
#[derive(Debug)]
pub struct TestBinTested<'a> {
    test_bin: TestBin<'a>,
    exe_time: Duration,
    pub coverage: CoverageCollection,
}

impl<'a> TestBin<'a> {
    pub fn new(bin_path: &'a Path, id: usize) -> Self {
        Self { id, bin_path }
    }

    // run the test and record the covered mutators and the time required to run the tests.
    pub fn run_test(
        self,
        progress: &mut Progress,
        mutations: &[BakedMutation],
    ) -> Fallible<TestBinTested<'a>> {
        let num_mutations = mutations.len();
        let test_start = Instant::now();

        progress.start_testsuite_unmutated(&self.bin_path, self.id)?;

        ::std::io::stdout().flush()?;

        // run test suite
        let mut command = Command::new(self.bin_path);
        command.env("MUTAGEN_MODE", "coverage");
        command.env("MUTAGEN_NUM_MUTATIONS", format!("{}", num_mutations));
        command.env("MUTAGEN_TESTSUITE", &self.bin_path);
        command.stdout(Stdio::null());
        let mut test_run = command.spawn()?;
        let status = test_run.wait()?;
        let exe_time = test_start.elapsed();

        let success = status.success();

        if !success {
            bail!("test suite fails. Retry after `cargo test` succeeds");
        }

        // read the coverage-file for this testsuite and delete it afterwards
        let coverage = {
            let coverage_file = comm::get_coverage_file()?;
            if !coverage_file.exists() {
                // no coverage file means that no mutations has been covered
                CoverageCollection::new_empty(num_mutations)
            } else {
                let coverage_hits = comm::read_items::<CoverageHit>(&coverage_file)?;
                // delete coverage file after the execution of this testsuite
                fs::remove_file(coverage_file)?;

                CoverageCollection::from_coverage_hits(num_mutations, &coverage_hits, &mutations)
            }
        };

        progress.finish_testsuite_unmutated(success, coverage.num_covered())?;

        Ok(TestBinTested {
            test_bin: self,
            coverage,
            exe_time,
        })
    }
}

impl<'a> TestBinTested<'a> {

    /// Checks if any mutation is covered.
    ///
    /// Returns false, if no mutation is covered by the testsuite
    pub fn coveres_any_mutation(&self) -> bool {
        self.coverage.num_covered() != 0
    }

    pub fn check_mutant(&self, mutation: &BakedMutation) -> Fallible<MutantStatus> {
        // run command and wait for its output
        let mut command = Command::new(self.test_bin.bin_path);
        command.env("MUTATION_ID", mutation.id().to_string());
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());
        let mut test_run = command.spawn()?;

        let wait_time = 5 * self.exe_time + Duration::from_millis(500);
        let timeout = test_run.wait_timeout(wait_time)?;

        Ok(match timeout {
            Some(status) => {
                if status.success() {
                    MutantStatus::Survived
                } else {
                    MutantStatus::Killed(status.code())
                }
            }
            None => {
                test_run.kill()?;
                MutantStatus::Timeout
            }
        })
    }
}
