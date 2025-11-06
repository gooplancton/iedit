use std::io::{BufWriter, Write};

mod edit_buffer;
mod line;
mod status;

use termion::cursor::Goto;

use crate::{
    Editor,
    editor::highlight::SyntaxHighlight,
    terminal::{CLEAR_BELOW_CURSOR, CLEAR_LINE, CURSOR_DOWN1, CURSOR_TO_COL1, H_BAR, UILayout},
};

pub struct Renderer<'editor, Term: Write> {
    term: BufWriter<&'editor mut Term>,
    ui: UILayout,
    syntax_highlight: Option<SyntaxHighlight>,
    horizontal_bar: String,
    tab_size: usize,
    is_first_render: bool,
}

impl<'term, Term: Write> Renderer<'term, Term> {
    pub fn new(
        term: &'term mut Term,
        ui: UILayout,
        tab_size: usize,
        syntax_highlight: Option<SyntaxHighlight>,
    ) -> Self {
        let horizontal_bar = str::repeat(H_BAR, ui.term_width as usize);

        Self {
            term: BufWriter::with_capacity(24 * 1024, term),
            ui,
            horizontal_bar,
            tab_size,
            syntax_highlight,
            is_first_render: true,
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
        self.add(termion::color::Reset.bg_str())?;

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

        Ok(())
    }

    #[inline]
    pub fn clear_line(&mut self) -> std::io::Result<()> {
        self.add(CLEAR_LINE)?;
        self.add(termion::color::Reset.bg_str())?;

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

        let cursor_rel_x = (editor.cursor.cur_x - editor.viewport.left_col) as u16
            + self.ui.ui_origin.0
            + 7u16 * editor.config.show_line_numbers as u16;
        let cursor_rel_y =
            (editor.cursor.cur_y - editor.viewport.top_line) as u16 + self.ui.ui_origin.1;

        self.add(termion::cursor::Goto(cursor_rel_x, cursor_rel_y).to_string())?;
        self.term.flush()?;
        self.is_first_render = false;

        Ok(())
    }
}
