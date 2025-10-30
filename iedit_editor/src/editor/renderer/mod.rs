use std::io::{BufWriter, Write};

mod edit_buffer;
mod legacy_line_renderer;
mod status;

use termion::cursor::Goto;

use crate::{
    Editor,
    terminal::{
        CLEAR_BELOW_CURSOR, CLEAR_LINE, CURSOR_DOWN1, CURSOR_TO_COL1, H_BAR, HIGHLIGHT_END,
        UILayout,
    },
};

pub struct Renderer<'editor, Term: Write> {
    term: BufWriter<&'editor mut Term>,
    ui: UILayout,
    horizontal_bar: String,
}

impl<'term, Term: Write> Renderer<'term, Term> {
    pub fn new(term: &'term mut Term, ui: UILayout) -> Self {
        let horizontal_bar = str::repeat(H_BAR, ui.term_width as usize);

        Self {
            term: BufWriter::new(term),
            ui,
            horizontal_bar,
        }
    }

    #[inline]
    pub fn reset_cursor(&mut self) -> std::io::Result<()> {
        write!(
            self.term,
            "{}",
            Goto(self.ui.ui_origin.0, self.ui.ui_origin.1)
        )?;

        self.add(CURSOR_TO_COL1)?;
        self.add(CLEAR_LINE)?;
        self.add(HIGHLIGHT_END)?;

        Ok(())
    }

    #[inline]
    pub fn add(&mut self, bytes: impl AsRef<[u8]>) -> std::io::Result<()> {
        self.term.write_all(bytes.as_ref())
    }

    #[inline]
    pub fn add_horizontal_bar(&mut self) -> std::io::Result<()> {
        self.term.write_all(self.horizontal_bar.as_ref())
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
