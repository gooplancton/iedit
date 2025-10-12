use std::cmp::{max, min};

use crate::editor::viewport::Viewport;

use super::Editor;

#[derive(Default)]
pub struct EditorState {
    pub viewport: Viewport,
    pub cursor_pos_x: usize,
    pub cursor_pos_y: usize,
    pub ideal_cursor_pos_x: usize,
    pub cursor_vel_x: isize,
    pub cursor_vel_y: isize,

    pub is_file_modified: bool,

    pub is_selecting: bool,
}

impl Editor {
    pub fn get_status_text(&self) -> String {
        let line = self.state.cursor_pos_y + 1;
        let col = self.state.cursor_pos_x + 1;
        let modified = if self.state.is_file_modified { "*" } else { "" };
        let total_lines = self.file_lines.len();
        format!(
            "{}{} | Ln {}, Col {} | {} lines",
            self.file_name, modified, line, col, total_lines
        )
    }
}
