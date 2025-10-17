use crate::{editor::Editor, line::EditorLine};

impl<TextLine: EditorLine> Editor<TextLine> {
    pub fn insert_char(&mut self, c: char) {
        let total_lines = self.file_lines.len();
        let (x, y) = self.state.get_cursor_pos_mut();
        let x = *x;

        if y.is_some_and(|y| *y == total_lines) {
            self.file_lines.push(TextLine::new());
        }

        let line = self.get_current_line_mut();
        if x >= line.len() {
            line.push_char(c);
        } else {
            line.insert_char_at(c, x);
        }

        self.move_cursor_right();
        self.state.is_file_modified = true;
    }

    pub fn insert_tab(&mut self) {
        for _ in 0..self.config.tab_size {
            self.insert_char(' ');
        }
    }

    pub fn delete_char(&mut self) {
        let (x, y) = self.state.get_cursor_pos();

        if x == 0 && y.is_some_and(|y| y == 0 || y >= self.file_lines.len()) {
            return;
        }

        let line = self.get_current_line_mut();

        if x > 0 && x <= line.len() {
            line.remove_char_at(x - 1);
            self.move_cursor_left();
        } else if x == 0 && y.is_some_and(|y| y > 0) {
            // Merge with previous line
            let y = y.unwrap();
            let prev_line_len = self.file_lines[y - 1].len();
            let mut current_line = self.file_lines.remove(y);
            self.file_lines[y - 1].merge_at_end(&mut current_line);
            self.move_cursor_up();
            self.state.cursor_pos_x = prev_line_len;
            self.state.set_ideal_cursor_pos_x();
        }

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
            line.delete_chars(start_x..end_x);
        } else {
            let (before_last_line, last_line) = self.file_lines.split_at_mut(end_y);
            before_last_line[start_y].delete_chars(start_x..);

            let after_end = &mut last_line[0];
            after_end.delete_chars(..end_x);

            before_last_line[start_y].merge_at_end(after_end);

            self.file_lines.drain(start_y + 1..=end_y);
        }

        self.state.cursor_pos_x = start_x;
        self.state.cursor_pos_y = start_y;
        self.state.ideal_cursor_pos_x = self.state.cursor_pos_x;
        self.state.selection_anchor = None;
        self.state.is_file_modified = true;
    }

    pub fn delete_word(&mut self) {
        let (x, y) = self.state.get_cursor_pos();

        if x == 0 && y.is_some_and(|y| y > 0) {
            return;
        }

        let current_line = self.get_current_line();
        let mut new_x = x;
        let is_current_char_alphanum = |x| {
            current_line
                .get_nth_char(x)
                .is_some_and(char::is_alphanumeric)
        };

        // Skip whitespace
        while new_x > 0 && !is_current_char_alphanum(new_x - 1) {
            new_x -= 1;
        }

        // Skip current word
        while new_x > 0 && is_current_char_alphanum(new_x - 1) {
            new_x -= 1;
        }

        if new_x < x {
            let old_x = x;
            let line = match y {
                Some(y) => &mut self.file_lines[y],
                None => &mut self.state.command_text,
            };
            line.delete_chars(new_x..old_x);
            let (x, _) = self.state.get_cursor_pos_mut();
            *x = new_x;
            self.state.set_ideal_cursor_pos_x();
            self.state.is_file_modified = true;
        }
    }

    pub fn insert_newline(&mut self) {
        let y = self.state.cursor_pos_y;
        let x = self.state.cursor_pos_x;

        if y == self.file_lines.len() {
            self.file_lines.push(TextLine::new());
        }
        let current_line = if x < self.file_lines[y].len() {
            self.file_lines[y].split_chars_off_at(x)
        } else {
            TextLine::new()
        };
        self.file_lines.insert(y + 1, current_line);
        self.state.cursor_pos_y += 1;
        self.state.cursor_pos_x = 0;
        self.state.ideal_cursor_pos_x = self.state.cursor_pos_x;
        self.state.is_file_modified = true;
    }
}
