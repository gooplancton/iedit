use std::cmp::{max, min};

use crate::editor::Editor;

pub struct Viewport {
    pub top_line: usize,
    pub bottom_line: usize,
    pub left_col: usize,
    pub right_col: usize,
    pub pre_scroll_top_line: usize,
    pub vertical_offset: isize,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            top_line: 0,
            bottom_line: usize::MAX,
            left_col: 0,
            right_col: usize::MAX,
            pre_scroll_top_line: 0,
            vertical_offset: 0,
        }
    }
}

impl Editor {
    pub fn adjust_viewport(&mut self) {
        let n_lines = self.config.n_lines as usize;
        let n_cols = (self.term_width as usize) - self.config.show_line_numbers as usize * 7;
        let vertical_margin = self.config.vertical_margin as usize;
        let y = self.state.cursor_pos_y;

        let top_limit = self.state.viewport.pre_scroll_top_line + vertical_margin;
        let bottom_limit = self.state.viewport.pre_scroll_top_line + n_lines - vertical_margin;

        let should_scroll_up = y < top_limit && self.state.cursor_vel_y < 0;
        let should_scroll_down = y > bottom_limit && self.state.cursor_vel_y > 0;
        let lines_below_viewport = self
            .file_lines
            .len()
            .saturating_sub(self.state.viewport.top_line + n_lines - 1);

        if should_scroll_up {
            let vertical_scroll = top_limit.saturating_sub(self.state.cursor_pos_y);
            self.state.viewport.pre_scroll_top_line = self
                .state
                .viewport
                .pre_scroll_top_line
                .saturating_sub(vertical_scroll);
            self.needs_full_rerender = true;
        } else if should_scroll_down && lines_below_viewport > 0 {
            let vertical_scroll = min(lines_below_viewport, y.saturating_sub(bottom_limit));
            self.state.viewport.pre_scroll_top_line += vertical_scroll;
            self.needs_full_rerender = true;
        }

        self.state.viewport.top_line = self.state.viewport.pre_scroll_top_line;

        let offset = self.state.viewport.vertical_offset;
        self.state.viewport.top_line = min(
            self.file_lines.len(),
            max(0, (self.state.viewport.top_line as isize) + offset) as usize,
        );
        self.state.viewport.bottom_line = self.state.viewport.top_line + n_lines;

        let left_limit = self.state.viewport.left_col + self.config.horizontal_margin as usize;
        let right_limit = self
            .state
            .viewport
            .right_col
            .saturating_sub(self.config.horizontal_margin as usize);

        if self.state.cursor_pos_x < left_limit && self.state.cursor_vel_x < 0 {
            let horizontal_scroll = left_limit.saturating_sub(self.state.cursor_pos_x);
            self.state.viewport.left_col = self
                .state
                .viewport
                .left_col
                .saturating_sub(horizontal_scroll);
            self.state.viewport.right_col = self.state.viewport.left_col + n_cols;
            self.needs_full_rerender = true;
        } else if self.state.cursor_pos_x > right_limit && self.state.cursor_vel_x > 0 {
            let horizontal_scroll = self.state.cursor_pos_x.saturating_sub(right_limit);
            self.state.viewport.right_col += horizontal_scroll;
            self.state.viewport.left_col = self.state.viewport.right_col.saturating_sub(n_cols);
            self.needs_full_rerender = true;
        }
    }
}
