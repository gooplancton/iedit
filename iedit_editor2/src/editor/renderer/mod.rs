use std::io::{Stdout, Write};

use termion::{
    cursor::{Goto, HideCursor},
    raw::RawTerminal,
};

pub struct Renderer {
    pub term: HideCursor<RawTerminal<Stdout>>,
    pub ui_origin: (u16, u16),
    pub term_width: u16,
    pub horizontal_bar: String,
    pub dirty_lines: Vec<usize>,
    pub needs_full_rerender: bool,
}

impl Renderer {
    pub fn reset_cursor(&mut self) -> std::io::Result<()> {
        write!(self.term, "{}", Goto(self.ui_origin.0, self.ui_origin.1))
    }
}
