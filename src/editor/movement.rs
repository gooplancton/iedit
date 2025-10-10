use super::Editor;

pub enum MovementDirection {
    Up,
    Down,
    Left,
    Right,
}

impl Editor {
    pub fn move_cursor_up(&mut self) {
        self.state.cursor_pos_y -= 1;
        self.state.cursor_file_row = self.state.cursor_file_row.saturating_sub(1);
    }
    pub fn move_cursor_down(&mut self) {
        self.state.cursor_pos_y += 1;
        self.state.cursor_file_row += 1;
    }
    pub fn move_cursor_left(&mut self) {
        self.state.cursor_pos_x -= 1;
        self.state.cursor_file_col = self.state.cursor_file_col.saturating_sub(1);
    }
    pub fn move_cursor_right(&mut self) {
        self.state.cursor_pos_x += 1;
        self.state.cursor_file_col += 1;
    }
    // others (eg. go to line, etc..)
}
