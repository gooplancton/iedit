use std::io::Write;

use iedit_document::CharacterEditable;

use crate::{
    Editor,
    editor::{highlight::SelectionHighlight, renderer::legacy_line_renderer::LineRenderer},
    terminal::{
        self, CLEAR_LINE, CURSOR_DOWN1, CURSOR_TO_COL1, EMPTY_CURSOR, HIGHLIGHT_END, V_BAR,
    },
};

impl Editor {
    fn render_line(&mut self, line_idx: usize) -> std::io::Result<()> {
        let line = &self.document.lines[line_idx];

        self.renderer.add(CLEAR_LINE)?;
        self.renderer.add(HIGHLIGHT_END)?;
        if self.config.show_line_numbers {
            let line_number_color = if line_idx == self.cursor.cur_y {
                termion::color::White.fg_str()
            } else {
                termion::color::LightBlack.fg_str()
            };

            write!(
                self.renderer.term,
                "{}{: >5}{} {}",
                line_number_color,
                line_idx + 1,
                termion::color::Reset.fg_str(),
                V_BAR,
            )?;
        }

        let display_start = self.viewport.left_col;
        let display_end =
            (self.viewport.left_col + self.renderer.term_width as usize).min(line.n_chars());
        let highlighted_range = self.cursor.get_highlighted_range();

        let selection_highlight = if let Some(highlighted_range) = highlighted_range {
             SelectionHighlight::new(line_idx, &highlighted_range)
        } else if self.cursor.cur_y == line_idx {
            SelectionHighlight::Range(self.cursor.cur_x, self.cursor.cur_x + 1)
        } else {
            SelectionHighlight::None
        };

        LineRenderer::new(line)
            .with_display_range(display_start..display_end)
            .with_selection_highlight(selection_highlight)
            .render_to(&mut self.renderer.term)?;

        let is_cursor_at_end_of_line =
            self.cursor.cur_y == line_idx && self.cursor.cur_x == line.n_chars();
        if is_cursor_at_end_of_line && self.cursor.selection_anchor.is_none() {
            self.renderer.add(terminal::EMPTY_CURSOR.as_bytes())?;
        }

        Ok(())
    }

    fn render_empty_line(&mut self, with_cursor: bool) -> std::io::Result<()> {
        self.renderer.add(CLEAR_LINE.as_bytes())?;
        self.renderer.add(HIGHLIGHT_END.as_bytes())?;

        if self.config.show_line_numbers {
            self.renderer
                .term
                .write_fmt(format_args!("{: >5} {}", " ", V_BAR))?;
        }
        let content = if with_cursor { EMPTY_CURSOR } else { "~" };
        self.renderer.add(content.as_bytes())?;

        Ok(())
    }

    pub fn render_edit_buffer(&mut self) -> std::io::Result<()> {
        let row_span_low = self.viewport.top_line;
        let row_span_high =
            (self.viewport.top_line + self.config.n_lines as usize).min(self.document.n_lines());

        for line_idx in row_span_low..row_span_high {
            let should_render_line = self.renderer.needs_full_rerender
                || line_idx == self.cursor.cur_y
                || line_idx == self.cursor.past_y
                || self.viewport.vertical_offset != 0
                || self.renderer.dirty_lines.contains(&line_idx);

            if should_render_line {
                self.render_line(line_idx)?;
            }

            self.renderer.add(CURSOR_DOWN1.as_bytes())?;
            self.renderer.add(CURSOR_TO_COL1.as_bytes())?;
        }

        let empty_lines = self.config.n_lines as usize - (row_span_high - row_span_low);
        for empty_line_idx in 0..empty_lines {
            let with_cursor = empty_line_idx == 0 && self.cursor.cur_y >= self.document.lines.len();
            self.render_empty_line(with_cursor)?;
            self.renderer.add(CURSOR_DOWN1.as_bytes())?;
            self.renderer.add(CURSOR_TO_COL1.as_bytes())?;
        }

        self.renderer.dirty_lines.truncate(0);
        self.renderer.needs_full_rerender = false;

        Ok(())
    }
}
