use std::{
    fs::OpenOptions,
    io::{Stdout, Write},
};

use termion::{
    cursor::HideCursor,
    raw::{IntoRawMode, RawTerminal},
};

use crate::editor::Editor;
use iedit_macros::ConfigParse;

#[derive(ConfigParse)]
pub struct EditorConfig {
    pub n_lines: u16,
    pub min_real_estate: u16,
    pub horizontal_margin: u16,
    pub vertical_margin: u16,
    pub show_line_numbers: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            n_lines: 0,
            min_real_estate: 10,
            horizontal_margin: 4,
            vertical_margin: 4,
            show_line_numbers: true,
        }
    }
}

impl EditorConfig {
    /// Automatically infers the optimal number of lines to display based on the terminal height
    /// and the starting position of the UI.
    ///
    /// Returns the scroll offset applied to the terminal to ensure enough space for the editor UI.
    pub fn set_default_n_lines(
        &mut self,
        term: &mut HideCursor<RawTerminal<Stdout>>,
        ui_start_y: u16,
    ) -> std::io::Result<u16> {
        term.suspend_raw_mode()?;

        let term_height = termion::terminal_size()?.1;
        let mut real_estate = term_height.saturating_sub(ui_start_y);
        let mut offset = self.min_real_estate.saturating_sub(real_estate);
        if offset > 0 {
            real_estate = self.min_real_estate;
            let newlines = "\n".repeat(real_estate as usize);
            write!(term, "{}{}", newlines, termion::cursor::Up(offset))?;
            term.flush()?;
        }

        // NOTE: 2 lines are reserved for the status bar
        self.n_lines = real_estate.saturating_sub(2);

        term.activate_raw_mode()?;

        Ok(offset)
    }
}
