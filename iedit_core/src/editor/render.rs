use std::{
    cmp::{max, min},
    io::Write,
};

use crate::{
    HIGHLIGHT,
    terminal::{self, CLEAR_LINE, CURSOR_DOWN1, CURSOR_TO_COL1, V_BAR},
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
        let cursor_col = self.state.cursor_pos_x;

        self.term.write_all(CLEAR_LINE.as_bytes())?;
        if self.config.show_line_numbers {
            self.term
                .write_fmt(format_args!("{: >5} {}", line_idx + 1, V_BAR))?;
        }

        let range_start = self.state.viewport.left_col;
        let range_end = self.state.viewport.right_col.min(line.len());

        let content = line.get(range_start..range_end).unwrap_or_default();
        let is_current_line = line_idx == self.state.cursor_pos_y;
        if is_current_line {
            let cursor_display_pos = min(content.len(), cursor_col.saturating_sub(range_start));

            let before_cursor = content.get(..cursor_display_pos).unwrap_or_default();
            self.term.write_all(before_cursor.as_bytes())?;

            let at_cursor = content.chars().nth(cursor_display_pos).unwrap_or(' ');
            self.term.write_fmt(HIGHLIGHT!(at_cursor))?;

            let after_cursor = content.get((cursor_display_pos + 1)..).unwrap_or_default();
            self.term.write_all(after_cursor.as_bytes())?;
        } else {
            self.term.write_all(content.as_bytes())?;
        }

        Ok(())
    }

    fn render_empty_line(&mut self) -> std::io::Result<()> {
        self.term.write_all(CLEAR_LINE.as_bytes())?;
        if self.config.show_line_numbers {
            self.term.write_fmt(format_args!("{: >5} {}", " ", V_BAR))?;
        }
        self.term.write_all("~".as_bytes())?;

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
        for _ in 0..empty_lines {
            self.render_empty_line()?;
            self.term.write_all(CURSOR_DOWN1.as_bytes())?;
            self.term.write_all(CURSOR_TO_COL1.as_bytes())?;
        }

        Ok(())
    }

    fn render_status(&mut self) -> std::io::Result<()> {
        self.term.write_all(CLEAR_LINE.as_bytes())?;
        self.term.write_all(self.horizontal_bar.as_bytes())?;
        self.term.write_all(CURSOR_DOWN1.as_bytes())?;
        self.term.write_all(CURSOR_TO_COL1.as_bytes())?;

        self.term
            .write_fmt(format_args!("{}", self.get_status_text()))?;
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
