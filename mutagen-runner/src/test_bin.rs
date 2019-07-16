use failure::{bail, Fallible};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use wait_timeout::ChildExt;

use super::MutantStatus;
use mutagen::BakedMutation;

/// wrapper around a test-binary that can be executed
pub struct TestBin<'a> {
    bin_path: &'a Path,
}

// wrapper around a test-binary, which has been run already and its runtime has been timed.
pub struct TestBinTimed<'a> {
    test_bin: TestBin<'a>,
    exe_time: Duration,
}

impl<'a> TestBin<'a> {
    pub fn new(bin_path: &'a Path) -> Self {
        Self { bin_path }
    }

    // run the test and record the time required.
    pub fn run_test(self) -> Fallible<TestBinTimed<'a>> {
        let test_start = Instant::now();

        // run test suite
        let mut command = Command::new(self.bin_path);
        command.stdout(Stdio::null());
        let mut test_run = command.spawn()?;
        let status = test_run.wait()?;

        if !status.success() {
            bail!("test suite fails. Retry after `cargo test` succeeds");
        }

        Ok(TestBinTimed {
            test_bin: self,
            exe_time: test_start.elapsed(),
        })
    }
}

impl<'a> TestBinTimed<'a> {
    pub fn check_mutant(&self, mutation: &BakedMutation) -> Fallible<MutantStatus> {
        // run command and wait for its output
        let mut command = Command::new(self.test_bin.bin_path);
        command.env("MUTATION_ID", mutation.id().to_string());
        command.stdout(Stdio::null());
        let mut test_run = command.spawn()?;

        let wait_time = 5 * self.exe_time + Duration::from_millis(500);
        let timeout = test_run.wait_timeout(wait_time)?;

        Ok(match timeout {
            Some(status) => {
                if status.success() {
                    MutantStatus::MutantSurvived
                } else {
                    MutantStatus::MutantKilled(status.code())
                }
            }
            None => {
                test_run.kill()?;
                MutantStatus::Timeout
            }
        })
    }
}
