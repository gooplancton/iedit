use std::{
    cmp::min,
    io::{self, BufWriter, Stdout, Write},
};

mod edit_buffer;
mod legacy_line_renderer;
mod status;

use termion::{
    cursor::{DetectCursorPos, Goto, HideCursor},
    raw::RawTerminal,
    terminal_size,
};

use crate::{
    Editor,
    terminal::{
        CLEAR_BELOW_CURSOR, CLEAR_LINE, CURSOR_DOWN1, CURSOR_TO_COL1, H_BAR, HIGHLIGHT_END,
    },
};

#[derive(Clone)]
pub struct UI {
    pub ui_origin: (u16, u16),
    pub term_width: u16,
    pub term_height: u16,
    pub editor_lines: u16,
    pub horizontal_bar: String,
}

impl UI {
    pub fn new(editor_lines: u16) -> io::Result<Self> {
        let (term_width, term_height) = terminal_size()?;
        let horizontal_bar = str::repeat(H_BAR, term_width as usize);

        Ok(UI {
            ui_origin: (1, 1), // Will be set properly by Renderer
            term_width,
            term_height,
            editor_lines,
            horizontal_bar,
        })
    }
}

pub struct Renderer<'editor, Term: Write> {
    term: BufWriter<&'editor mut Term>,
    ui: UI,
}

impl<'term, Term: Write> Renderer<'term, Term> {
    pub fn new(term: &'term mut Term, ui: UI) -> Self {
        Self {
            term: BufWriter::new(term),
            ui,
        }
    }

    #[inline]
    pub fn reset_cursor(&mut self) -> std::io::Result<()> {
        write!(
            self.term,
            "{}",
            Goto(self.ui.ui_origin.0, self.ui.ui_origin.1)
        )
    }

    #[inline]
    pub fn add(&mut self, bytes: impl AsRef<[u8]>) -> std::io::Result<()> {
        self.term.write_all(bytes.as_ref())
    }

    #[inline]
    pub fn add_horizontal_bar(&mut self) -> std::io::Result<()> {
        self.term.write_all(self.ui.horizontal_bar.as_ref())
    }

    #[inline]
    pub fn next_line(&mut self) -> std::io::Result<()> {
        self.add(CURSOR_DOWN1)?;
        self.add(CURSOR_TO_COL1)?;
        self.add(CLEAR_LINE)?;
        self.add(HIGHLIGHT_END)?;

        Ok(())
    }

    pub fn cleanup(&mut self) -> std::io::Result<()> {
        self.reset_cursor()?;
        self.add(CLEAR_BELOW_CURSOR)?;

        Ok(())
    }

    pub fn initialize_ui(
        term: &mut HideCursor<RawTerminal<Stdout>>,
        ui: &mut UI,
    ) -> io::Result<()> {
        let ui_origin = term.cursor_pos()?;
        let ui_start_y = ui_origin.1;
        let max_scroll = if ui.editor_lines == 0 {
            ui.term_height / 2
        } else {
            ui.editor_lines + 2
        };

        let mut real_estate = ui.term_height.saturating_sub(ui_start_y);
        let offset = max_scroll.saturating_sub(real_estate);
        if offset > 0 {
            real_estate = min(max_scroll, ui.term_height);
            let newlines = "\n".repeat(real_estate as usize);
            write!(term, "{}{}", newlines, termion::cursor::Up(offset))?;
            term.flush()?;
        }

        ui.editor_lines = real_estate.saturating_sub(2);
        ui.ui_origin = (ui_origin.0, ui_origin.1 - offset);

        term.activate_raw_mode()?;
        Ok(())
    }

    pub fn render<'editor>(&mut self, editor: &'editor Editor) -> std::io::Result<()>
    where
        'term: 'editor,
    {
        self.reset_cursor()?;
        editor.render_edit_buffer(self)?;
        editor.render_status(self)?;
        self.term.flush()?;

        Ok(())
    }
}
