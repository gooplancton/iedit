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
        let cursor_col = self.state.cursor_file_col;
        let mut term_width = self.term_width;

        self.term.write_all(CLEAR_LINE.as_bytes())?;
        if self.config.show_line_numbers {
            self.term
                .write_fmt(format_args!("{: >5} {}", line_idx + 1, V_BAR))?;
            term_width -= 7;
        }

        let (range_start, range_end, scroll_x) =
            self.compute_x_frame(line.len(), term_width as usize);

        self.state.cursor_pos_x -= scroll_x as i16;

        let content = line.get(range_start..range_end).unwrap_or_default();
        let is_current_line = line_idx == self.state.cursor_file_row;
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

    fn render_content(&mut self) -> std::io::Result<()> {
        let (row_span_low, row_span_high, scroll_y) = self.compute_y_frame();

        self.state.cursor_pos_y -= scroll_y as i16;

        for line_idx in row_span_low..row_span_high {
            self.render_line(line_idx)?;
            self.term.write_all(CURSOR_DOWN1.as_bytes())?;
            self.term.write_all(CURSOR_TO_COL1.as_bytes())?;
        }

        Ok(())
    }

    fn render_status(&mut self) -> std::io::Result<()> {
        let total_lines = self.file_lines.len();
        let current_line_len = self.file_lines[self.state.cursor_file_row].len();

        self.term.write_all(CLEAR_LINE.as_bytes())?;
        self.term.write_all(self.horizontal_bar.as_bytes())?;
        self.term.write_all(CURSOR_DOWN1.as_bytes())?;
        self.term.write_all(CURSOR_TO_COL1.as_bytes())?;

        self.term.write_fmt(format_args!(
            "Line {}/{}, Col: {}/{}",
            self.state.cursor_file_row, total_lines, self.state.cursor_file_col, current_line_len,
        ))?;
        self.term.write_all(CURSOR_DOWN1.as_bytes())?;
        self.term.write_all(CURSOR_TO_COL1.as_bytes())?;

        Ok(())
    }

    pub fn cleanup(&mut self) -> std::io::Result<()> {
        self.term
            .write_fmt(format_args!("{}", self.reset_cursor_seq))?;
        self.term
            .write_all(terminal::CLEAR_BELOW_CURSOR.as_bytes())?;

        Ok(())
    }

    fn compute_y_frame(&self) -> (usize, usize, isize) {
        let snap_y_low = 0;
        let snap_y_high = self.file_lines.len();

        let frame_y_low = self
            .state
            .cursor_file_row
            .saturating_sub(self.state.cursor_pos_y as usize);
        let frame_y_high = min(frame_y_low + self.config.n_lines as usize, snap_y_high);

        let wiggle_y_low = frame_y_low - snap_y_low;
        let wiggle_y_high = snap_y_high - frame_y_low;

        let pos_y = max(
            frame_y_low,
            min(frame_y_high, self.state.cursor_pos_y as usize),
        );
        let mut scroll_y: isize = 0;

        let distance_to_y_start = pos_y - frame_y_low;
        let distance_to_y_end = frame_y_high - pos_y;

        let margin_y = self.config.vertical_margin as usize;

        if self.state.cursor_vel_y > 0 && distance_to_y_end < margin_y && wiggle_y_high > 0 {
            scroll_y = min(margin_y - distance_to_y_end, wiggle_y_high) as isize;
        } else if self.state.cursor_vel_y < 0 && distance_to_y_start < margin_y && wiggle_y_low > 0
        {
            scroll_y = max(
                0isize - (margin_y - distance_to_y_start) as isize,
                0isize - wiggle_y_low as isize,
            );
        }

        (
            frame_y_low + scroll_y as usize,
            frame_y_high + scroll_y as usize,
            scroll_y,
        )
    }

    fn compute_x_frame(&self, line_len: usize, term_width: usize) -> (usize, usize, isize) {
        let snap_x_low = 0;
        let snap_x_high = line_len;

        let frame_x_low = self
            .state
            .cursor_file_col
            .saturating_sub(self.state.cursor_pos_x as usize);
        let frame_x_high = min(frame_x_low + term_width, snap_x_high);

        let wiggle_x_low = frame_x_low - snap_x_low;
        let wiggle_x_high = snap_x_high - frame_x_low;

        let pos_x = max(
            frame_x_low,
            min(frame_x_high, self.state.cursor_pos_x as usize),
        );
        let mut scroll_x: isize = 0;

        let distance_to_x_start = pos_x - frame_x_low;
        let distance_to_x_end = frame_x_high - pos_x;

        let margin_x = self.config.horizontal_margin as usize;

        if self.state.cursor_vel_x > 0 && distance_to_x_end < margin_x && wiggle_x_high > 0 {
            scroll_x = min(margin_x - distance_to_x_end, wiggle_x_high) as isize;
        } else if self.state.cursor_vel_x < 0 && distance_to_x_start < margin_x && wiggle_x_low > 0
        {
            scroll_x = max(
                0isize - (margin_x - distance_to_x_start) as isize,
                0isize - wiggle_x_low as isize,
            );
        }

        (
            frame_x_low + scroll_x as usize,
            frame_x_high + scroll_x as usize,
            scroll_x,
        )
    }
}

