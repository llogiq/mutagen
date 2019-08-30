//! Custom implementation of printing progress of the cargo-mutagen runner.
//!
//! This module contains a progress bar similar to the one cargo uses.
//! If the output is not a terminal or the terminal is too small, no progress bar is shown.
//! The progress bar tries to be adaptive as possible and only uses a single line in every case.
//!
//! The main challenges is to be able to continue writing to the line above the progress bar.

use console::Term;
use failure::{format_err, Fallible};
use std::io::Write;

use mutagen_core::comm::{BakedMutation, MutantStatus};

/// Print progress during mutation testing
pub struct Progress {
    term: Term,
    term_width: usize,
    show_progress: bool,
    num_mutations: usize,
    current_log_str: Option<String>,
}

impl Progress {
    pub fn new(num_mutations: usize) -> Self {
        let term = Term::stdout();
        let term_width = term.size().1 as usize;
        let show_progress = term.is_term() && term_width > 20;

        Self {
            term,
            term_width,
            show_progress,
            num_mutations,
            current_log_str: None,
        }
    }

    /// indicate that a test-run begins.
    ///
    /// The information about the mutation is logged to the console.
    /// A call to `finish_mutation` should follow a call to this function
    pub fn start_mutation(&mut self, m: &BakedMutation) -> Fallible<()> {
        let mutant_log_string = mutation_log_string(m);

        if self.show_progress {
            self.term.clear_line()?;
        }

        write!(&self.term, "{} ... ", &mutant_log_string)?;

        // write progress bar
        if self.show_progress {
            writeln!(&self.term)?;
            self.write_progress_bar(m)?;

            // save log-str for later
            self.current_log_str = Some(mutant_log_string);
        }

        Ok(())
    }

    /// indicate that a mutation started with `start_mutation` has been finished.
    ///
    /// The status is printed and progress bar is updated
    pub fn finish_mutation(&mut self, status: MutantStatus) -> Fallible<()> {
        if self.show_progress {
            let log_str = self
                .current_log_str
                .take()
                .ok_or_else(|| format_err!("calling report_status without starting a mutation"))?;

            let term_with = self.term.size().1 as usize;
            let log_str_len = log_str.len();
            let log_str_lines = 1 + log_str_len / term_with;

            // clear progress bar
            self.term.clear_line()?;
            self.term.clear_last_lines(log_str_lines)?;

            writeln!(&self.term, "{} ... {}", log_str, status)?;
        } else {
            writeln!(&self.term, "{}", status)?;
        }

        Ok(())
    }

    /// indicate that mutation-testing is finished
    ///
    /// clears the progress-bar
    pub fn finish(self) -> Fallible<()> {
        if self.show_progress {
            self.term.clear_line()?;
            writeln!(&self.term)?;
        }
        Ok(())
    }

    fn write_progress_bar(&self, m: &BakedMutation) -> Fallible<()> {
        let m_id = m.id();

        let current_total_string = format!("{}/{}", m_id, self.num_mutations);
        let action_name = console::style(format!("{:>12}", "Test Mutants")).bold();

        let main_part_len = self.term_width.min(80);

        // construct progress bar
        let bar_width = main_part_len - 18 - current_total_string.len();
        let mut bar_pos = bar_width * m_id / self.num_mutations;
        if bar_pos == bar_width {
            bar_pos -= 1;
        }
        let bar1 = "=".repeat(bar_pos);
        let bar2 = " ".repeat(bar_width - bar_pos - 1);

        // construct status details right to progress bar, if there is space for it
        let fn_name = m.context_description_in_brackets();
        let mut action_details = format!(": {}{}", m.source_file().display(), fn_name);
        let space_after_main_bar = self.term_width - main_part_len;
        if space_after_main_bar < 10 {
            action_details = "".to_owned();
        } else if space_after_main_bar < action_details.len() {
            action_details = format!("{:.*}...", space_after_main_bar - 3, action_details);
        }

        write!(
            &self.term,
            "{} [{}>{}] {}{}\r",
            action_name, bar1, bar2, current_total_string, action_details
        )?;

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
