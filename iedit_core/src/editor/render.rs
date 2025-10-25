use std::io::Write;

use crate::{
    line::{LineRenderer, SelectionHighlight},
    terminal::{self, CLEAR_LINE, CURSOR_DOWN1, CURSOR_TO_COL1, HIGHLIGHT_END, V_BAR},
};

use super::Editor;

impl Editor {
    pub fn render(&mut self) -> std::io::Result<()> {
        self.reset_cursor()?;
        self.render_content()?;
        self.render_status()?;
        self.term.flush()?;

        Ok(())
    }

    fn render_line(&mut self, line_idx: usize) -> std::io::Result<()> {
        let line = &self.file_lines[line_idx];

        self.term.write_all(CLEAR_LINE.as_bytes())?;
        self.term.write_all(HIGHLIGHT_END.as_bytes())?;
        if self.config.show_line_numbers {
            let line_number_color = if line_idx == self.state.cursor_pos_y {
                termion::color::White.fg_str()
            } else {
                termion::color::LightBlack.fg_str()
            };

            write!(
                self.term,
                "{}{: >5}{} {}",
                line_number_color,
                line_idx + 1,
                termion::color::Reset.fg_str(),
                V_BAR,
            )?;
        }

        let display_start = self.state.viewport.left_col;
        let display_end = self.state.viewport.right_col.min(line.len());
        let highlighted_range = self.get_highlighted_range();
        let selection_highlight = SelectionHighlight::new(line_idx, &highlighted_range);

        LineRenderer::new(line)
            .with_display_range(display_start..display_end)
            .with_selection_highlight(selection_highlight)
            .render_to(&mut self.term)?;

        let is_cursor_at_end_of_line =
            self.state.cursor_pos_y == line_idx && self.state.cursor_pos_x == line.len();
        if is_cursor_at_end_of_line && self.state.selection_anchor.is_none() {
            self.term.write_all(terminal::EMPTY_CURSOR.as_bytes())?;
        }

        Ok(())
    }

    fn render_empty_line(&mut self, with_cursor: bool) -> std::io::Result<()> {
        self.term.write_all(CLEAR_LINE.as_bytes())?;
        self.term.write_all(HIGHLIGHT_END.as_bytes())?;

        if self.config.show_line_numbers {
            self.term.write_fmt(format_args!("{: >5} {}", " ", V_BAR))?;
        }
        let content = if with_cursor {
            terminal::EMPTY_CURSOR
        } else {
            "~"
        };
        self.term.write_all(content.as_bytes())?;

        Ok(())
    }

    fn render_content(&mut self) -> std::io::Result<()> {
        let row_span_low = self.state.viewport.top_line;
        let row_span_high = self.state.viewport.bottom_line.min(self.file_lines.len());

        for line_idx in row_span_low..row_span_high {
            let should_render_line = self.needs_full_rerender
                || self.dirty_lines.contains(&line_idx)
                || line_idx == self.state.cursor_pos_y
                || line_idx == self.state.cursor_previous_pos_y;

            if should_render_line {
                self.render_line(line_idx)?;
            }

            self.term.write_all(CURSOR_DOWN1.as_bytes())?;
            self.term.write_all(CURSOR_TO_COL1.as_bytes())?;
        }

        let empty_lines = self.config.n_lines as usize - (row_span_high - row_span_low);
        for empty_line_idx in 0..empty_lines {
            let with_cursor =
                empty_line_idx == 0 && self.state.cursor_pos_y >= self.file_lines.len();
            self.render_empty_line(with_cursor)?;
            self.term.write_all(CURSOR_DOWN1.as_bytes())?;
            self.term.write_all(CURSOR_TO_COL1.as_bytes())?;
        }

        self.dirty_lines.truncate(0);
        self.needs_full_rerender = false;

        Ok(())
    }

    fn render_temp_message(&mut self) -> std::io::Result<()> {
        self.term.write_all(self.temp_message.as_bytes())?;

        self.term.write_all(CURSOR_DOWN1.as_bytes())?;
        self.term.write_all(CURSOR_TO_COL1.as_bytes())?;

        self.term.write_all(CLEAR_LINE.as_bytes())?;

        self.temp_message.truncate(0);

        Ok(())
    }

    fn render_status(&mut self) -> std::io::Result<()> {
        self.term.write_all(CLEAR_LINE.as_bytes())?;
        self.term.write_all(self.horizontal_bar.as_bytes())?;
        self.term.write_all(CURSOR_DOWN1.as_bytes())?;
        self.term.write_all(CURSOR_TO_COL1.as_bytes())?;

        self.term.write_all(CLEAR_LINE.as_bytes())?;
        self.term.write_all("  ".as_bytes())?;

        if !self.temp_message.is_empty() {
            return self.render_temp_message();
        }

        let content = if !self.state.is_editing_content() {
            &self.state.command_text
        } else {
            self.update_status_text();
            &self.state.status_text
        };

        let mut renderer =
            LineRenderer::new(content).with_display_range(0..self.term_width as usize);

        if !self.state.is_editing_content() {
            self.term.write_all(":".as_bytes())?;
            let x = self.state.cmd_cursor_pos_x;
            let selection_highlight = SelectionHighlight::Range(x, x + 1);
            renderer = renderer.with_selection_highlight(selection_highlight);
        }

        renderer.render_to(&mut self.term)?;

        let is_cursor_at_end_of_line = !self.state.is_editing_content()
            && self.state.cmd_cursor_pos_x == self.state.command_text.len();
        if is_cursor_at_end_of_line {
            self.term.write_all(terminal::EMPTY_CURSOR.as_bytes())?;
        }

        self.term.write_all(CURSOR_DOWN1.as_bytes())?;
        self.term.write_all(CURSOR_TO_COL1.as_bytes())?;

        self.term.write_all(CLEAR_LINE.as_bytes())?;

        Ok(())
    }

    pub fn cleanup(&mut self) -> std::io::Result<()> {
        self.reset_cursor()?;
        self.term
            .write_all(terminal::CLEAR_BELOW_CURSOR.as_bytes())?;

        Ok(())
    }
}
