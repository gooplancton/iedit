use std::{
    cmp::min,
    io::{self, Write},
};

use iedit_document::CharacterIndexable;
use termion::cursor;

use crate::editor::renderer::Renderer;

impl<'term, Term: Write> Renderer<'term, Term> {
    pub fn render_popup<'editor>(
        &mut self,
        lines: &'editor [impl AsRef<str>],
        origin_pos: (u16, u16),
        selected_idx: Option<u16>,
    ) -> io::Result<()>
    where
        'term: 'editor,
    {
        let popup_height = min(self.ui.editor_lines as usize, lines.len() + 2) as u16;
        let popup_width = lines
            .iter()
            .map(|line| line.as_ref().n_chars())
            .max()
            .unwrap_or_default()
            + 2;
        let popup_origin_y = self.ui.ui_origin.1 + origin_pos.1;
        let popup_origin_x = self.ui.ui_origin.0
            + (if origin_pos.0 + popup_width as u16 > self.ui.term_width {
                self.ui.term_width - popup_width as u16
            } else {
                origin_pos.0
            });

        self.add(cursor::Goto(popup_origin_x, popup_origin_y).to_string())?;
        self.add(format!("╭{}╮", "─".repeat(popup_width - 2)))?;

        for line_idx in 1..popup_height - 1 {
            let is_selected = selected_idx.is_some_and(|idx| idx == line_idx - 1);

            self.add(cursor::Goto(popup_origin_x, line_idx + popup_origin_y).to_string())?;
            self.add("│")?;
            if is_selected {
                self.add(termion::color::LightBlack.bg_str())?;
            }
            self.add(lines[line_idx as usize - 1].as_ref())?;
            self.add(
                " ".repeat(popup_width - lines[line_idx as usize - 1].as_ref().n_chars() - 2),
            )?;
            self.add(termion::color::Reset.bg_str())?;
            self.add("│")?;
        }

        self.add(cursor::Goto(popup_origin_x, popup_height - 1 + popup_origin_y).to_string())?;
        self.add(format!("╰{}╯", "─".repeat(popup_width - 2)))?;

        Ok(())
    }
}
