//! Custom implementation of printing progress of the cargo-mutagen runner.
//!
//! This module contains a progress bar similar to the one cargo uses.
//! If the output is not a terminal or the terminal is too small, no progress bar is shown.
//! The progress bar tries to be adaptive as possible and only uses a single line in every case.
//!
//! The main challenges is to be able to continue writing to the line above the progress bar.

use console::Term;
use failure::Fallible;
use std::io::Write;

/// Print progress during mutation testing
pub struct ProgressBar {
    term: Term,
    term_width: usize,
    show_progress: bool,
    total: usize,
    current_log_str: Option<String>,
    current_bar_state: Option<ProgressBarState>,
}

pub struct ProgressBarState {
    pub action: &'static str,
    pub current: usize,
    pub action_details: String,
}

impl ProgressBar {
    pub fn new() -> Self {
        let term = Term::stdout();
        let term_width = term.size().1 as usize;
        let show_progress = term.is_term() && term_width > 20;

        Self {
            term,
            term_width,
            show_progress,
            total: 0,
            current_log_str: None,
            current_bar_state: None,
        }
    }

    /// re-sets to total actions to perform
    pub fn set_total(&mut self, total: usize) {
        self.total = total;
    }

    pub fn shows_progress(&self) -> bool {
        self.show_progress
    }

    /// prints some text to stdout.
    ///
    /// If the progress bar is shown, the text is printed above the progress bar.
    /// The next call to `println` will continue writing the line started by this function.
    pub fn print(&mut self, s: String) -> Fallible<()> {
        if self.show_progress {
            self.term.clear_line()?;
        }

        // TODO: allow multiple print-calls and newlines
        assert!(
            self.current_log_str.is_none(),
            "consecutive calls to ProgressBar::print are currently not supported"
        );
        assert!(
            !s.contains('\n'),
            "newlines are currently not supported in ProgressBar::print"
        );

        write!(&self.term, "{}", &s)?;

        if self.show_progress {
            writeln!(&self.term)?;

            self.write_progress_bar()?;

            self.current_log_str = Some(s);
        }
        Ok(())
    }

    /// prints a line to stdout.
    ///
    /// If the progress bar is shown, the line is printed above the progress bar.
    pub fn println(&mut self, s: &str) -> Fallible<()> {
        if self.show_progress {
            self.term.clear_line()?;

            if let Some(log_str) = self.current_log_str.take() {
                let log_str_lines = 1 + (log_str.len() + 1) / self.term_width;
                self.term.clear_last_lines(log_str_lines)?;
                writeln!(&self.term, "{}{}", log_str, s)?;
            } else {
                writeln!(&self.term, "{}", s)?;
            }

            self.write_progress_bar()?;
        } else {
            writeln!(&self.term, "{}", s)?;
        }

        self.current_log_str = None;

        Ok(())
    }

    /// clears the progress bar
    pub fn clear_bar(&mut self) -> Fallible<()> {
        if let Some(_) = self.current_bar_state.take() {
            self.term.clear_line()?;
        }
        Ok(())
    }

    /// finish the progress bar
    ///
    /// clears the progress-indicator
    pub fn finish(self) -> Fallible<()> {
        if self.show_progress {
            self.term.clear_line()?;
            writeln!(&self.term)?;
        }
        Ok(())
    }

    /// updates the state of the progress bar and draws the new state if the progress is shown
    pub fn set_state(&mut self, bar: ProgressBarState) -> Fallible<()> {
        if self.show_progress {
            self.current_bar_state = Some(bar);
            self.write_progress_bar()?;
        }
        Ok(())
    }

    fn write_progress_bar(&self) -> Fallible<()> {
        if let Some(bar) = &self.current_bar_state {
            if self.total == 0 {
                return Ok(());
            }
            let current_total_string = format!("{}/{}", bar.current, self.total);
            let action_name = console::style(format!("{:>12}", bar.action)).bold();

            let main_part_len = self.term_width.min(80);

            // construct progress bar
            let bar_width = main_part_len - 18 - current_total_string.len();
            let mut bar_pos = bar_width * bar.current / self.total;
            if bar_pos == bar_width {
                bar_pos -= 1;
            }
            let bar1 = "=".repeat(bar_pos);
            let bar2 = " ".repeat(bar_width - bar_pos - 1);

            // print status details right to progress bar, if there is space for it
            let mut action_details = bar.action_details.to_owned();
            action_details = format!(": {}", action_details);
            let space_after_main_bar = self.term_width - main_part_len;
            if space_after_main_bar < 10 {
                action_details = "".to_owned();
            } else if space_after_main_bar < action_details.len() {
                action_details = format!("{:.*}...", space_after_main_bar - 5, action_details);
            }

            write!(
                &self.term,
                "{} [{}>{}] {}{}\r",
                action_name, bar1, bar2, current_total_string, action_details
            )?;
        }

        Ok(())
    }
}
