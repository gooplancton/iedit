use crate::editor::Editor;

impl Editor {
    pub fn insert_char(&mut self, c: char) {
        let mut y = self.state.cursor_pos_y;
        let x = self.state.cursor_pos_x;

        let line = &mut self.file_lines[y];
        if x > line.len() {
            line.push(c);
        } else {
            line.insert(x, c);
        }
        self.state.cursor_pos_x += 1;
        self.state.ideal_cursor_pos_x = self.state.cursor_pos_x;
        self.state.is_file_modified = true;
    }

    pub fn delete_char(&mut self) {
        let y = self.state.cursor_pos_y;
        let x = self.state.cursor_pos_x;

        if x == 0 && y == 0 {
            return;
        }

        let line = &mut self.file_lines[y];

        if x > 0 && x <= line.len() {
            line.remove(x - 1);
            self.state.cursor_pos_x -= 1;
        } else if x == 0 && y > 0 {
            // Merge with previous line
            let prev_line_len = self.file_lines[y - 1].len();
            let current_line = self.file_lines.remove(y);
            self.file_lines[y - 1].push_str(&current_line);
            self.state.cursor_pos_y -= 1;
            self.state.cursor_pos_x = prev_line_len;
        }

        self.state.ideal_cursor_pos_x = self.state.cursor_pos_x;
        self.state.is_file_modified = true;
    }

    pub fn insert_newline(&mut self) {
        let mut y = self.state.cursor_pos_y;
        let x = self.state.cursor_pos_x;
        let current_line = if x < self.file_lines[y].len() {
            self.file_lines[y].split_off(x)
        } else {
            String::new()
        };
        self.file_lines.insert(y + 1, current_line);
        self.state.cursor_pos_y += 1;
        self.state.cursor_pos_x = 0;
        self.state.ideal_cursor_pos_x = self.state.cursor_pos_x;
        self.state.is_file_modified = true;
    }
}
