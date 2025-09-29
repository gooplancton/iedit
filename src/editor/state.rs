use std::cmp::{max, min};

use super::Editor;

#[derive(Default)]
pub struct EditorState {
    pub status_text: String,

    pub cursor_file_row: usize,
    pub cursor_file_col: usize,
    pub cursor_pos_x: i16,
    pub cursor_pos_y: i16,
    pub cursor_vel_x: i16,
    pub cursor_vel_y: i16,

    pub is_selecting: bool,
}

impl Editor {
    pub fn clamp_cursor(&mut self) {
        let max_y = self.file_lines.len();
        let max_x = self.file_lines[self.state.cursor_file_row].len();

        self.state.cursor_file_row = min(max_y, self.state.cursor_file_row);
        self.state.cursor_file_col = min(max_x, self.state.cursor_file_col);
    }
}
