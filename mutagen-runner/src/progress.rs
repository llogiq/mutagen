//! Custom implementation of printing progress of the cargo-mutagen runner.
//!
//! This module contains a progress bar similar to the one cargo uses.
//! If the output is not a terminal or the terminal is too small, no progress bar is shown.
//! The progress bar tries to be adaptive as possible and only uses a single line in every case.
//!
//! The main challenges is to be able to continue writing to the line above the progress bar.

use failure::Fallible;

use std::path::Path;

use mutagen_core::comm::{BakedMutation, MutantStatus};

use super::progress_bar::{ProgressBar, ProgressBarState};

/// Print progress during mutation testing
pub struct Progress {
    bar: ProgressBar,
}

impl Progress {
    pub fn new(num_mutations: usize) -> Self {
        Self {
            bar: ProgressBar::new(num_mutations),
        }
    }

    // start the section that runs the test suites unmutated
    pub fn section_testsuite_unmutated(&mut self) -> Fallible<()> {
        self.bar.println("")?;
        self.bar.println("Tests without mutations")?;
        Ok(())
    }

    // start the section of test-runs for each mutation
    pub fn section_mutants(&mut self) -> Fallible<()> {
        self.bar.println("")?;
        self.bar.println("Mutants")?;
        Ok(())
    }

    pub fn start_testsuite_unmutated(&mut self, bin: &Path) -> Fallible<()> {
        let log_string = format!("{} ... ", bin.display());
        self.bar.print(log_string)?;

        if self.bar.shows_progress() {
            let bar = ProgressBarState {
                action: "Run Tests",
                current: 0,
                action_details: format!("{}", bin.display()),
            };

            self.bar.write_progress_bar(bar)?;
        }

        Ok(())
    }

    pub fn finish_testsuite_unmutated(&mut self, ok: bool) -> Fallible<()> {
        self.bar.println(if ok { "ok" } else { "FAILED" })
    }

    /// indicate that a test-run begins.
    ///
    /// The information about the mutation is logged to the console.
    /// A call to `finish_mutation` should follow a call to this function
    pub fn start_mutation(&mut self, m: &BakedMutation) -> Fallible<()> {
        let mut mutant_log_string = mutation_log_string(m);
        mutant_log_string += " ... ";

        self.bar.print(mutant_log_string)?;

        // write progress bar
        if self.bar.shows_progress() {
            let action_details = format!(
                ": {}{}",
                m.source_file().display(),
                m.context_description_in_brackets(),
            );
            let bar = ProgressBarState {
                action: "Test Mutants",
                current: m.id(),
                action_details: action_details,
            };
            self.bar.write_progress_bar(bar)?;
        }

        Ok(())
    }

    /// indicate that a mutation started with `start_mutation` has been finished.
    ///
    /// The status is printed and progress bar is updated
    pub fn finish_mutation(&mut self, status: MutantStatus) -> Fallible<()> {
        self.bar.println(&format!("{}", status))?;
        Ok(())
    }

    /// indicate that mutation-testing is finished
    ///
    /// clears the progress-bar
    pub fn finish(self) -> Fallible<()> {
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
