use crate::{CharacterEditable, Document, DocumentLine, edit::EditResult};

impl Document {
    pub fn insert_char_at(&mut self, (x, y): (usize, usize), ch: char) -> EditResult {
        let line = self.get_or_add_line(y)?;
        line.insert_char_at(ch, x);

        Some((x + 1, y))
    }

    pub fn insert_string_at(
        &mut self,
        (x, y): (usize, usize),
        string: impl AsRef<str>,
    ) -> EditResult {
        let line = self.get_or_add_line(y)?;
        line.insert_str_at(x, string.as_ref());

        Some((x + string.as_ref().n_chars(), y))
    }

    pub fn insert_lines_at(&mut self, (x, y): (usize, usize), lines: Vec<String>) -> EditResult {
        if lines.is_empty() {
            return Some((x, y));
        }

        let n_lines = lines.len();
        let last_line_len = self
            .lines
            .get(y + lines.len() - 1)
            .map(|s| s.n_chars())
            .unwrap_or(0);

        // Handle the case where we're inserting in the middle of an existing line
        let is_middle_of_line = self.lines.get(y).is_some_and(|line| x < line.n_chars()); // Fix: Use n_chars()

        if is_middle_of_line {
            let line = self.lines.remove(y);
            let (left, right) = line.split_chars_at(x);

            // Create the new first line
            let mut first_line = String::with_capacity(left.n_chars() + lines[0].n_chars());
            first_line.push_str(left);
            first_line.push_str(&lines[0]);

            // Build the remaining lines to insert
            let mut new_lines = Vec::with_capacity(lines.len());
            new_lines.push(first_line);
            new_lines.extend_from_slice(&lines[1..]);

            // Modify the last inserted line to include the right part
            let last_line_len = if let Some(last_line) = new_lines.last_mut() {
                last_line.push_str(right);
                last_line.n_chars()
            } else {
                0
            };

            self.lines.splice(y..=y, new_lines);

            let final_y = y + lines.len() - 1;
            let final_x = last_line_len - right.n_chars();
            return Some((final_x, final_y));
        }

        // Simple case: insert at beginning of line or beyond existing content
        if y < self.lines.len() {
            self.lines.splice(y..y, lines);
        } else {
            self.lines.extend(lines);
        }

        Some((last_line_len, y + n_lines - 1))
    }

    pub fn insert_newline_at(&mut self, (x, y): (usize, usize)) -> EditResult {
        let line = self.get_or_add_line(y)?;

        let first_nonwhitespace_x = line
            .iter_chars()
            .position(|char| !char.is_whitespace())
            .unwrap_or_default();

        let mut to_append = String::from(line.get_chars(0..first_nonwhitespace_x));

        if x < line.n_chars() {
            to_append.merge_at_end(&mut line.split_chars_off_at(x).trim_start().to_owned())
        }

        self.lines.insert(y + 1, to_append);

        Some((first_nonwhitespace_x, y + 1))
    }
}
