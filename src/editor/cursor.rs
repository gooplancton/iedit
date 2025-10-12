use std::cmp::{max, min};

use super::Editor;

pub enum MovementDirection {
    Up,
    Down,
    Left,
    Right,
}

impl Editor {
    pub fn move_cursor_up(&mut self) {
        self.state.cursor_pos_y = self.state.cursor_pos_y.saturating_sub(1);
    }
    pub fn move_cursor_down(&mut self) {
        self.state.cursor_pos_y += 1;
    }
    pub fn move_cursor_left(&mut self) {
        self.state.cursor_pos_x = self.state.cursor_pos_x.saturating_sub(1);
    }
    pub fn move_cursor_right(&mut self) {
        self.state.cursor_pos_x += 1;
    }

    pub fn clamp_cursor(&mut self) {
        let max_y = self.file_lines.len() - 1;
        self.state.cursor_pos_y = max(0, min(max_y, self.state.cursor_pos_y));

        let max_x = self.file_lines[self.state.cursor_pos_y].len();
        self.state.cursor_pos_x = max(0, min(max_x, self.state.cursor_pos_x));
    }

    // others (eg. go to line, etc..)
}
