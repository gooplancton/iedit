use std::{
    cmp::min,
    io::{self, Write},
};

use iedit_document::CharacterIndexable;
use termion::cursor;

use crate::editor::renderer::Renderer;

impl<'editor, Term: Write> Renderer<'editor, Term> {
    /// this will wrap the provided lines in a frame and render it in the top-right corner of the ui
    pub fn render_popup(&mut self, lines: &'editor [&'editor str]) -> io::Result<()> {
        let popup_height = min(self.ui.editor_lines as usize - 1, lines.len() + 2) as u16;
        let popup_width = lines
            .iter()
            .map(|line| line.n_chars())
            .max()
            .unwrap_or_default()
            + 2;
        let popup_origin_x = (self.ui.term_width as usize).saturating_sub(popup_width) as u16;

        self.add(cursor::Goto(popup_origin_x, self.ui.ui_origin.1).to_string())?;
        self.add(format!("╭{}╮", "─".repeat(popup_width - 2)))?;

        for line_idx in 1..popup_height - 1 {
            self.add(cursor::Goto(popup_origin_x, line_idx + self.ui.ui_origin.1).to_string())?;
            self.add("│")?;
            self.add(lines[line_idx as usize - 1])?;
            self.add(" ".repeat(popup_width - lines[line_idx as usize - 1].n_chars() - 2))?;
            self.add("│")?;
        }

        self.add(cursor::Goto(popup_origin_x, popup_height - 1 + self.ui.ui_origin.1).to_string())?;
        self.add(format!("╰{}╯", "─".repeat(popup_width - 2)))?;

        Ok(())
    }
}
