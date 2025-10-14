use std::{
    cmp::{max, min},
    io::Write,
};

use crate::{
    editor::selection::SelectionHighlight,
    terminal::{self, CLEAR_LINE, CURSOR_DOWN1, CURSOR_TO_COL1, HIGHLIGHT_END, V_BAR},
};

use super::Editor;

impl Editor {
    pub fn render(&mut self) -> std::io::Result<()> {
        self.term
            .write_fmt(format_args!("{}", self.reset_cursor_seq))?;
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
            self.term
                .write_fmt(format_args!("{: >5} {}", line_idx + 1, V_BAR))?;
        }

        let range_start = self.state.viewport.left_col;
        let range_end = self.state.viewport.right_col.min(line.len());

        let content = line.get(range_start..range_end).unwrap_or_default();
        let highlighted_range = self.get_highlighted_range();
        let selection_highlight =
            SelectionHighlight::from_line_idx_and_selection_range(line_idx, &highlighted_range);
        let highlighted_content = selection_highlight.highlight_chars(content);

        self.term.write_all(highlighted_content.as_str().as_bytes());

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
            self.render_line(line_idx)?;
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

        Ok(())
    }

    fn render_status(&mut self) -> std::io::Result<()> {
        self.update_status_text();

        self.term.write_all(CLEAR_LINE.as_bytes())?;
        self.term.write_all(self.horizontal_bar.as_bytes())?;
        self.term.write_all(CURSOR_DOWN1.as_bytes())?;
        self.term.write_all(CURSOR_TO_COL1.as_bytes())?;

        self.term.write_all(CLEAR_LINE.as_bytes())?;
        self.term.write_all(self.state.status_text.as_bytes())?;
        self.term.write_all(CURSOR_DOWN1.as_bytes())?;
        self.term.write_all(CURSOR_TO_COL1.as_bytes())?;

        self.term.write_all(CLEAR_LINE.as_bytes())?;

        Ok(())
    }

    pub fn cleanup(&mut self) -> std::io::Result<()> {
        self.term
            .write_fmt(format_args!("{}", self.reset_cursor_seq))?;
        self.term
            .write_all(terminal::CLEAR_BELOW_CURSOR.as_bytes())?;

        Ok(())
    }
}
