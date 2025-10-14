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
        self.state.ideal_cursor_pos_x = self.state.cursor_pos_x
    }
    pub fn move_cursor_right(&mut self) {
        self.state.cursor_pos_x += 1;
        self.state.ideal_cursor_pos_x = self.state.cursor_pos_x
    }

    pub fn clamp_cursor(&mut self) {
        let max_y = self.file_lines.len();
        self.state.cursor_pos_y = max(0, min(max_y, self.state.cursor_pos_y));

        let max_x = self
            .file_lines
            .get(self.state.cursor_pos_y)
            .map_or(0, |line| line.len());
        self.state.cursor_pos_x = max(0, min(self.state.ideal_cursor_pos_x, max_x));
    }

    pub fn move_cursor_word_left(&mut self) {
        if self.state.cursor_pos_x == 0 {
            if self.state.cursor_pos_y > 0 {
                self.move_cursor_up();
                self.state.cursor_pos_x = self.get_current_line().len();
            }
        } else {
            let current_line = self.get_current_line();
            let mut new_x = self.state.cursor_pos_x - 1;
            let is_current_char_whitespace = |x| {
                current_line
                    .chars()
                    .nth(x)
                    .map_or(false, char::is_whitespace)
            };

            // Skip whitespace
            while new_x > 0 && is_current_char_whitespace(new_x) {
                new_x -= 1;
            }

            // Skip current word
            while new_x > 0 && !is_current_char_whitespace(new_x) {
                new_x -= 1;
            }

            self.state.cursor_pos_x = new_x;
        }

        self.state.ideal_cursor_pos_x = self.state.cursor_pos_x;
    }

    pub fn move_cursor_word_right(&mut self) {
        let current_line = self.get_current_line();
        let line_len = current_line.len();

        if self.state.cursor_pos_x >= line_len {
            if self.state.cursor_pos_y < self.file_lines.len() - 1 {
                self.move_cursor_down();
                self.state.cursor_pos_x = 0;
            }
        } else {
            let mut new_x = self.state.cursor_pos_x;
            let is_current_char_whitespace = |x| {
                current_line
                    .chars()
                    .nth(x)
                    .map_or(false, char::is_whitespace)
            };

            // Skip current word
            while new_x < line_len && !is_current_char_whitespace(new_x) {
                new_x += 1;
            }

            // Skip whitespace
            while new_x < line_len && is_current_char_whitespace(new_x) {
                new_x += 1;
            }

            self.state.cursor_pos_x = new_x;
        }

        self.state.ideal_cursor_pos_x = self.state.cursor_pos_x;
    }
}
