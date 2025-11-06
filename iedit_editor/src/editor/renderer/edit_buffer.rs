use std::io::Write;

use crate::{
    Editor,
    editor::{
        highlight::SelectionHighlight,
        renderer::{Renderer, line::LineRenderer},
    },
    terminal::{EMPTY_CURSOR, V_BAR},
};

impl Editor {
    pub fn render_line<'renderer, 'term, Term: Write>(
        &self,
        renderer: &'renderer mut Renderer<'term, Term>,
        line_idx: usize,
    ) -> std::io::Result<()> {
        renderer.clear_line()?;

        let line = &self.document.lines[line_idx];

        let mut ui_width = self.ui.term_width as usize;

        if self.config.show_line_numbers {
            let line_number_color = if line_idx == self.cursor.cur_y {
                termion::color::White.fg_str()
            } else {
                termion::color::LightBlack.fg_str()
            };

            renderer.add(format!(
                "{}{: >5}{} {}",
                line_number_color,
                line_idx + 1,
                termion::color::Reset.fg_str(),
                V_BAR,
            ))?;

            ui_width = ui_width.saturating_sub(7);
        }

        let display_start = self.viewport.left_col;
        let display_end = (self.viewport.left_col + ui_width).min(line.len());

        let highlighted_range = self.cursor.get_highlighted_range();

        let mut line_renderer = LineRenderer::new(
            line.as_ref(),
            (display_start, display_end),
            &mut renderer.term,
            renderer.tab_size,
        );

        if let Some(highlighted_range) = highlighted_range {
            let highlight = SelectionHighlight::new(line_idx, &highlighted_range);
            line_renderer.add_selection_highlight(highlight);
        };

        if self.cursor.cur_y == line_idx {
            line_renderer.add_cursor(self.cursor.cur_x);
        }

        line_renderer.render()?;

        Ok(())
    }

    pub fn render_empty_line<'renderer, 'term, Term: Write>(
        &self,
        renderer: &'renderer mut Renderer<'term, Term>,
        with_cursor: bool,
    ) -> std::io::Result<()> {
        renderer.clear_line()?;

        if self.config.show_line_numbers {
            renderer.add(format!("{: >5} {}", " ", V_BAR))?
        }
        let content = if with_cursor { EMPTY_CURSOR } else { "~" };
        renderer.add(content)?;

        Ok(())
    }

    pub fn render_edit_buffer<'renderer, 'term, Term: Write>(
        &self,
        renderer: &'renderer mut Renderer<'term, Term>,
    ) -> std::io::Result<()> {
        let row_span_low = self.viewport.top_line;
        let row_span_high =
            (self.viewport.top_line + self.ui.editor_lines as usize).min(self.document.n_lines());

        for line_idx in row_span_low..row_span_high {
            let should_render_line = self.needs_full_rerender
                || line_idx == self.cursor.cur_y
                || line_idx == self.cursor.past_y
                || self.document.is_line_dirty(line_idx);

            if should_render_line {
                self.render_line(renderer, line_idx)?;
            }

            renderer.next_line()?;
        }

        let empty_lines = self.ui.editor_lines as usize - (row_span_high - row_span_low);
        for empty_line_idx in 0..empty_lines {
            let with_cursor = empty_line_idx == 0 && self.cursor.cur_y >= self.document.lines.len();
            self.render_empty_line(renderer, with_cursor)?;

            renderer.next_line()?;
        }

        Ok(())
    }
}
