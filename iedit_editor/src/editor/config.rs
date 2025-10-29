use std::{
    cmp::min,
    io::{Stdout, Write},
};

use termion::{cursor::HideCursor, raw::RawTerminal};

use iedit_macros::ConfigParse;

#[derive(ConfigParse)]
pub struct EditorConfig {
    pub n_lines: u16,
    pub horizontal_margin: u16,
    pub vertical_margin: u16,
    pub tab_size: u16,
    pub show_line_numbers: bool,
    pub show_keybindings: bool,
    pub confirm_quit_unsaved_changes: bool,
    pub edit_debounce_time_secs: u64,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            n_lines: 0,
            horizontal_margin: 4,
            tab_size: 4,
            vertical_margin: 4,
            show_line_numbers: true,
            show_keybindings: true,
            confirm_quit_unsaved_changes: true,
            edit_debounce_time_secs: 1,
        }
    }
}

impl EditorConfig {
}
