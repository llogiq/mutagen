//! Custom implementation of printing progress of the cargo-mutagen runner.
//!
//! This module contains a progress bar similar to the one cargo uses.
//! If the output is not a terminal or the terminal is too small, no progress bar is shown.
//! The progress bar tries to be adaptive as possible and only uses a single line in every case.
//!
//! The main challenges is to be able to continue writing to the line above the progress bar.
//! The output to the terminal should look identical to piped output but contains a progress bar.

use anyhow::Result;

use std::path::Path;
use std::time::Duration;

use mutagen_core::comm::{BakedMutation, MutantStatus};

use super::progress_bar::{ProgressBar, ProgressBarState};

/// Print progress during mutation testing
pub struct Progress {
    num_mutations: usize,
    num_covered: usize,
    tested_mutations: usize,
    bar: ProgressBar,
}

impl Progress {
    pub fn new(num_mutations: usize) -> Self {
        Self {
            num_mutations,
            num_covered: 0,
            tested_mutations: 0,
            bar: ProgressBar::new(),
        }
    }

    /// Print summary information after the compilation of the test binaries.
    pub fn summary_compile(&mut self, num_mutations: usize, num_testsuites: usize) -> Result<()> {
        self.bar.println("")?;
        self.bar
            .println(&format!("Total mutations: {}", num_mutations))?;

        self.bar.set_total(num_testsuites);

        Ok(())
    }

    /// Start the section that runs the test suites unmutated.
    pub fn section_testsuite_unmutated(&mut self, num_tests: usize) -> Result<()> {
        self.bar.println("")?;
        self.bar.println(&format!("Run {} tests", num_tests))?;
        Ok(())
    }

    /// start the section of test-runs for each mutation
    pub fn section_mutants(&mut self) -> Result<()> {
        self.bar.println("")?;
        self.bar
            .println(&format!("Test {} Mutants", self.num_mutations))?;
        Ok(())
    }

    /// start the section of the
    pub fn section_summary(&mut self) -> Result<()> {
        self.bar.println("")?;
        self.bar.clear_bar()?;
        Ok(())
    }

    /// indicate the start of a run of a single testsuite without mutations
    pub fn start_testsuite_unmutated(&mut self, bin: &Path, id: usize) -> Result<()> {
        let log_string = format!("{} ... ", bin.display());
        self.bar.print(log_string)?;

        if self.bar.shows_progress() {
            let bar = ProgressBarState {
                action: "Run Tests",
                current: id + 1,
                action_details: format!("{}", bin.display()),
            };

            self.bar.set_state(bar)?;
        }

        Ok(())
    }

    /// indicate the end of a run of a single testsuite and display the result.
    pub fn finish_testsuite_unmutated(&mut self, ok: bool, num_covered: usize) -> Result<()> {
        if ok && num_covered > 0 {
            self.bar.println(&format!(
                "ok ({}/{} covered)",
                num_covered, self.num_mutations
            ))
        } else if ok && num_covered == 0 {
            self.bar.println("ok (NOTHING COVERED)")
        } else {
            self.bar.println("FAILED")
        }
    }

    /// print a summary after the testsuites have been run, especially coverage information.
    pub fn summary_testsuite_unmutated(&mut self, num_covered: usize) -> Result<()> {
        self.num_covered = num_covered;
        self.bar.set_total(num_covered);

        self.bar.println("")?;
        self.bar.println(&format!(
            "Mutations covered: {}/{}",
            self.num_covered, self.num_mutations
        ))
    }

    /// indicate that a test-run of a covered mutation begins.
    ///
    /// The information about the mutation is logged to the console.
    /// A call to `finish_mutation` should follow a call to this function
    pub fn start_mutation_covered(&mut self, m: &BakedMutation) -> Result<()> {
        let mut mutant_log_string = mutation_log_string(m);
        mutant_log_string += " ... ";

        self.bar.print(mutant_log_string)?;

        self.tested_mutations += 1;

        // write progress bar
        if self.bar.shows_progress() {
            let action_details = format!(
                "{}{}",
                m.source_file().display(),
                m.context_description_in_brackets(),
            );
            let bar = ProgressBarState {
                action: "Test Mutants",
                current: self.tested_mutations,
                action_details,
            };
            self.bar.set_state(bar)?;
        }

        Ok(())
    }

    pub fn skip_mutation_uncovered(&mut self, m: &BakedMutation) -> Result<()> {
        self.bar.println(&format!(
            "{} ... {}",
            mutation_log_string(m),
            MutantStatus::NotCovered
        ))
    }

    /// indicate that a mutation started with `start_mutation` has been finished.
    ///
    /// The status is printed and progress bar is updated
    pub fn finish_mutation(&mut self, status: MutantStatus) -> Result<()> {
        self.bar.println(&format!("{}", status))?;
        Ok(())
    }

    /// indicate that mutation-testing is finished
    ///
    /// clears the progress-bar
    pub fn finish(mut self, mutagen_time: Duration) -> Result<()> {
        let rounded_time = Duration::from_secs(mutagen_time.as_secs());
        self.bar.println(&format!(
            "Total time: {}",
            ::humantime::format_duration(rounded_time)
        ))?;
        self.bar.finish()?;
        Ok(())
    }
}

/// Generate a string used for logging
fn mutation_log_string(m: &BakedMutation) -> String {
    format!(
        "{}: {}, {}, at {}@{}{}",
        m.id(),
        m.mutator_name(),
        m.mutation_description(),
        m.source_file().display(),
        m.location_in_file(),
        m.context_description_in_brackets(),
    )
}
