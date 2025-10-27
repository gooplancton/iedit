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

        Some((x + string.as_ref().len(), y))
    }

    // NOTE: half of this was written by deepseek, review
    pub fn insert_lines_at(&mut self, (x, y): (usize, usize), lines: Vec<String>) -> EditResult {
        if lines.is_empty() {
            return Some((x, y));
        }

        let n_lines = lines.len();
        let last_line_len = self
            .lines
            .get(y + lines.len() - 1)
            .map(|s| s.len())
            .unwrap_or(0);

        // Handle the case where we're inserting in the middle of an existing line
        let is_middle_of_line = self.lines.get(y).is_some_and(|line| x < line.len());

        if is_middle_of_line {
            // TODO: check off-by-1
            let line = self.lines.remove(y);
            let (left, right) = line.split_chars_at(x);

            // Create the new first line
            let mut first_line = String::with_capacity(left.len() + lines[0].len());
            first_line.push_str(left);
            first_line.push_str(&lines[0]);

            // Build the remaining lines to insert
            let mut new_lines = Vec::with_capacity(lines.len());
            new_lines.push(first_line);

            // Add middle lines directly
            new_lines.extend_from_slice(&lines[1..]);

            // Modify the last inserted line to include the right part
            let last_line_len = if let Some(last_line) = new_lines.last_mut() {
                last_line.push_str(right);
                last_line.len()
            } else {
                0
            };

            // Replace the current line and insert additional lines
            self.lines.splice(y..=y, new_lines);

            let final_y = y + lines.len() - 1;
            let final_x = last_line_len - right.len();
            return Some((final_x, final_y));
        }

        // Simple case: insert at beginning of line or beyond existing content
        if y < self.lines.len() {
            self.lines.splice(y..y, lines);
        } else {
            // Beyond current document bounds, just append
            self.lines.extend(lines);
        }

        Some((last_line_len, y + n_lines - 1))
    }

    pub fn insert_newline_at(&mut self, (x, y): (usize, usize)) -> EditResult {
        let line = self.get_or_add_line(y)?;

        let current_line = if x < line.len() {
            line.split_chars_off_at(x)
        } else {
            String::new()
        };

        self.lines.insert(y + 1, current_line);

        Some((0, y + 1))
    }
}
