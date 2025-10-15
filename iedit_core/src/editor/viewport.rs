use crate::editor::{EditorLine, Editor};

pub struct Viewport {
    pub top_line: usize,
    pub bottom_line: usize,
    pub left_col: usize,
    pub right_col: usize,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            top_line: 0,
            bottom_line: usize::MAX,
            left_col: 0,
            right_col: usize::MAX,
        }
    }
}

impl<TextLine: EditorLine> Editor<TextLine> {
    pub fn adjust_viewport(&mut self) {
        let n_lines = self.config.n_lines as usize;
        let n_cols = (self.term_width as usize) - self.config.show_line_numbers as usize * 7;

        let top_limit = self.state.viewport.top_line + self.config.vertical_margin as usize;
        let bottom_limit = self
            .state
            .viewport
            .bottom_line
            .saturating_sub(self.config.vertical_margin as usize);

        if self.state.cursor_pos_y < top_limit && self.state.cursor_vel_y < 0 {
            let vertical_scroll = top_limit.saturating_sub(self.state.cursor_pos_y);
            self.state.viewport.top_line =
                self.state.viewport.top_line.saturating_sub(vertical_scroll);
            self.state.viewport.bottom_line = self.state.viewport.top_line + n_lines;
        } else if self.state.cursor_pos_y > bottom_limit && self.state.cursor_vel_y > 0 {
            let vertical_scroll = self.state.cursor_pos_y.saturating_sub(bottom_limit);
            self.state.viewport.bottom_line += vertical_scroll;
            self.state.viewport.top_line = self.state.viewport.bottom_line.saturating_sub(n_lines);
        }

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
        } else if self.state.cursor_pos_x > right_limit && self.state.cursor_vel_x > 0 {
            let horizontal_scroll = self.state.cursor_pos_x.saturating_sub(right_limit);
            self.state.viewport.right_col += horizontal_scroll;
            self.state.viewport.left_col = self.state.viewport.right_col.saturating_sub(n_cols);
        }
    }
}
