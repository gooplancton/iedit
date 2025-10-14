use crate::editor::Editor;

impl Editor {
    pub fn insert_char(&mut self, c: char) {
        let mut y = self.state.cursor_pos_y;
        let x = self.state.cursor_pos_x;

        if y == self.file_lines.len() {
            self.file_lines.push(String::new());
        }

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

        if (x == 0 && y == 0) || y >= self.file_lines.len() {
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

    pub fn delete_selection(&mut self) {
        let ((start_x, start_y), (mut end_x, mut end_y)) = self.get_highlighted_range();
        if end_y >= self.file_lines.len() {
            end_y = self.file_lines.len() - 1;
            end_x = self.file_lines[end_y].len();
        }

        if start_y == end_y {
            let line = &mut self.file_lines[start_y];
            line.drain(start_x..end_x);
        } else {
            let after_end_len = self.file_lines[end_y].len() - end_x;
            let after_end = &self.file_lines[end_y][end_x..].as_ptr();
            self.file_lines[start_y].drain(start_x..);
            // SAFETY: after_end is valid for after_end_len bytes as it is a slice of a String
            let after_end_str = unsafe {
                std::str::from_utf8_unchecked(std::slice::from_raw_parts(*after_end, after_end_len))
            };
            self.file_lines[start_y].push_str(after_end_str);
            self.file_lines.drain(start_y + 1..=end_y);
        }

        self.state.cursor_pos_x = start_x;
        self.state.cursor_pos_y = start_y;
        self.state.ideal_cursor_pos_x = self.state.cursor_pos_x;
        self.state.selection_anchor = None;
        self.state.is_file_modified = true;
    }

    pub fn insert_newline(&mut self) {
        let mut y = self.state.cursor_pos_y;
        let x = self.state.cursor_pos_x;

        if y == self.file_lines.len() {
            self.file_lines.push(String::new());
        }
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
