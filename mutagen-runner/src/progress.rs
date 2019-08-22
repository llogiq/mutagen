use console::Term;
use failure::{format_err, Fallible};
use std::io::Write;

use mutagen_core::comm::{BakedMutation, MutantStatus};

/// Print progress during mutation testing
pub struct Progress {
    term: Term,
    num_mutations: usize,
    current_log_str: Option<String>,
}

impl Progress {
    pub fn new(num_mutations: usize) -> Self {
        Self {
            term: Term::stdout(),
            num_mutations,
            current_log_str: None,
        }
    }

    /// indicate that a test-run begins.
    ///
    /// The information about the mutation is logged to the console.
    /// A call to `finish_mutation` should follow a call to this function
    pub fn start_mutation(&mut self, m: &BakedMutation) -> Fallible<()> {
        let mutant_log_string = m.log_string();
        let m_id = m.id();

        if self.term.is_term() {
            self.term.clear_line()?;
        }

        write!(&self.term, "{} ... ", &mutant_log_string)?;

        // write progress bar
        if self.term.is_term() {
            writeln!(&self.term)?;
            let progress_bar = format!(
                "{:.*}>",
                60 * m_id / self.num_mutations,
                "============================================================",
            );
            writeln!(
                &self.term,
                "{} [{:60}] {}/{}",
                console::style(format!("{:>13}", "Test Mutants")).bold(),
                progress_bar,
                m_id,
                self.num_mutations
            )?;
            self.term.move_cursor_up(1)?;

            // save log-str for later
            self.current_log_str = Some(mutant_log_string);
        }

        Ok(())
    }

    /// indicate that a mutation started with `start_mutation` has been finished.
    ///
    /// The status is printed and progress bar is updated
    pub fn finish_mutation(&mut self, status: MutantStatus) -> Fallible<()> {
        if self.term.is_term() {
            let log_str = self
                .current_log_str
                .take()
                .ok_or_else(|| format_err!("calling report_status without starting a mutation"))?;

            self.term.move_cursor_up(1)?;
            self.term.clear_line()?;
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
        if self.term.is_term() {
            self.term.clear_line()?;
            writeln!(&self.term)?;
        }
        Ok(())
    }
}
