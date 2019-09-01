use failure::{bail, Fallible};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use wait_timeout::ChildExt;

use mutagen_core::comm::{BakedMutation, MutantStatus};

use super::Progress;

/// wrapper around a test-binary that can be executed
#[derive(Debug)]
pub struct TestBin<'a> {
    pub bin_path: &'a Path,
}

// wrapper around a test-binary, which has been run already and its runtime has been timed.
#[derive(Debug)]
pub struct TestBinTimed<'a> {
    test_bin: TestBin<'a>,
    exe_time: Duration,
}

impl<'a> TestBin<'a> {
    pub fn new(bin_path: &'a Path) -> Self {
        Self { bin_path }
    }

    // run the test and record the time required.
    pub fn run_test(
        self,
        progress: &mut Progress,
        num_mutations: usize,
    ) -> Fallible<TestBinTimed<'a>> {
        let test_start = Instant::now();

        progress.start_testsuite_unmutated(&self.bin_path)?;

        ::std::io::stdout().flush()?;

        // run test suite
        let mut command = Command::new(self.bin_path);
        command.env("MUTAGEN_MODE", "coverage");
        command.env("MUTAGEN_NUM_MUTATIONS", format!("{}", num_mutations));
        command.stdout(Stdio::null());
        let mut test_run = command.spawn()?;
        let status = test_run.wait()?;
        let exe_time = test_start.elapsed();

        let success = status.success();

        progress.finish_testsuite_unmutated(success)?;

        if !success {
            bail!("test suite fails. Retry after `cargo test` succeeds");
        }

        Ok(TestBinTimed {
            test_bin: self,
            exe_time,
        })
    }
}

impl<'a> TestBinTimed<'a> {
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
