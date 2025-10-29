use std::{
    cmp::min,
    io::{self, BufWriter, Stdout, Write},
};

mod edit_buffer;
mod legacy_line_renderer;
mod status;

use termion::{
    cursor::{DetectCursorPos, Goto, HideCursor},
    raw::{IntoRawMode, RawTerminal},
    terminal_size,
};

use crate::{
    Editor,
    editor::config::EditorConfig,
    terminal::{CLEAR_BELOW_CURSOR, H_BAR},
};

pub struct Renderer {
    pub term: BufWriter<HideCursor<RawTerminal<Stdout>>>,
    pub ui_origin: (u16, u16),
    pub editor_lines: u16,
    pub term_width: u16,
    pub term_height: u16,
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

    pub fn new(editor_lines: u16) -> io::Result<Self> {
        let (term_width, term_height) = terminal_size()?;
        let horizontal_bar = str::repeat(H_BAR, term_width as usize);
        let mut term = HideCursor::from(std::io::stdout().into_raw_mode()?);
        let mut ui_origin = term.cursor_pos()?;

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

        // NOTE: 2 lines are reserved for the status bar
        let editor_lines = real_estate.saturating_sub(2);

        term.activate_raw_mode()?;
        ui_origin.1 -= offset;

        Ok(Renderer {
            term: BufWriter::new(term),
            ui_origin,
            term_width,
            term_height,
            editor_lines,
            horizontal_bar,
            dirty_lines: vec![],
            needs_full_rerender: true,
        })
    }
    // pub fn from_editor_config(&)
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
