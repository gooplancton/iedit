#![allow(dead_code, unused)]

use std::{
    cmp::min,
    io::{self, Stdout, Write, stdout},
    ops::DerefMut,
    os::fd::AsFd,
};

use termion::{
    cursor::{DetectCursorPos, HideCursor},
    raw::{IntoRawMode, RawTerminal},
    terminal_size,
};

pub static CURSOR_UP1: &str = "\x1b[1A";
pub static CURSOR_DOWN1: &str = "\x1b[1B";
pub static CURSOR_RIGHT1: &str = "\x1b[1C";
pub static CURSOR_LEFT1: &str = "\x1b[1D";
pub static CURSOR_TO_LINE1: &str = "\x1b[1;1H";
pub static CLEAR_LINE: &str = "\x1b[2K";
pub static CLEAR_BELOW_CURSOR: &str = "\x1b[J";
pub static CURSOR_TO_COL1: &str = "\r";
pub static SAVE_CURSOR: &str = "\x1b[s";
pub static RESTORE_CURSOR: &str = "\x1b[u";
pub static HIDE_CURSOR: &str = "\x1b[?25l";
pub static SHOW_CURSOR: &str = "\x1b[?25h";
pub static CLEAR_SCREEN: &str = "\x1b[2J";
pub static HIGHLIGHT_START: &str = "\x1b[7m";
pub static HIGHLIGHT_END: &str = "\x1b[0m";
pub static EMPTY_CURSOR: &str = "\x1b[7m \x1b[0m";
pub static H_BAR: &str = "─";
pub static V_BAR: char = '│';

#[derive(Clone)]
pub struct UILayout {
    pub ui_origin: (u16, u16),
    pub term_width: u16,
    pub term_height: u16,
    pub editor_lines: u16,
}

impl UILayout {
    pub fn new<W: Write + AsFd>(
        editor_lines: u16,
        term: &mut impl DerefMut<Target = RawTerminal<W>>,
    ) -> io::Result<UILayout> {
        let (term_width, term_height) = terminal_size()?;
        let ui_origin = term.cursor_pos()?;

        let ui_start_y = ui_origin.1;
        let max_scroll = if editor_lines == 0 {
            term_height / 2
        } else {
            editor_lines + 2
        };

        let mut real_estate = term_height.saturating_sub(ui_start_y);
        let offset = max_scroll.saturating_sub(real_estate);
        if offset > 0 {
            real_estate = min(max_scroll, term_height);
            let newlines = "\n".repeat(real_estate as usize);
            write!(term, "{}{}", newlines, termion::cursor::Up(offset))?;
            term.flush()?;
        }

        let editor_lines = real_estate.saturating_sub(2);
        let ui_origin = (ui_origin.0, ui_origin.1 - offset);

        term.activate_raw_mode()?;

        Ok(UILayout {
            editor_lines,
            ui_origin,
            term_height,
            term_width,
        })
    }
}
