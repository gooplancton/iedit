use std::io::{BufWriter, Stdout, Write};

mod edit_buffer;
mod legacy_line_renderer;
mod status;

use termion::{
    cursor::{Goto, HideCursor},
    raw::RawTerminal,
};

use crate::{Editor, terminal::CLEAR_BELOW_CURSOR};

pub struct Renderer {
    pub term: BufWriter<HideCursor<RawTerminal<Stdout>>>,
    pub ui_origin: (u16, u16),
    pub term_width: u16,
    pub horizontal_bar: String,
    pub dirty_lines: Vec<usize>,
    pub needs_full_rerender: bool,
}

impl Renderer {
    #[inline]
    pub fn reset_cursor(&mut self) -> std::io::Result<()> {
        write!(self.term, "{}", Goto(self.ui_origin.0, self.ui_origin.1))
    }

    #[inline]
    pub fn add(&mut self, bytes: impl AsRef<[u8]>) -> std::io::Result<()> {
        self.term.write_all(bytes.as_ref())
    }

    #[inline]
    pub fn add_horizontal_bar(&mut self) -> std::io::Result<()> {
        self.term.write_all(self.horizontal_bar.as_ref())
    }
}

impl Editor {
    pub fn render(&mut self) -> std::io::Result<()> {
        self.renderer.reset_cursor()?;
        self.render_edit_buffer()?;
        self.render_status()?;
        self.renderer.term.flush()?;

        Ok(())
    }

    pub fn cleanup(&mut self) -> std::io::Result<()> {
        self.renderer.reset_cursor()?;
        self.renderer.add(CLEAR_BELOW_CURSOR)?;

        Ok(())
    }
}
