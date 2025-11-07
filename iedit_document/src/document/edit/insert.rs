use crate::Document;
use crate::document::edit::EditResult;
use crate::line::{CharacterIndexable, DocumentLine};

impl Document {
    pub fn insert_char_at(&mut self, (x, y): (usize, usize), ch: char) -> EditResult {
        let line = self.get_or_add_line(y)?;
        line.insert(x, ch);

        Some((x + 1, y))
    }

    pub fn insert_string_at(
        &mut self,
        (x, y): (usize, usize),
        string: impl AsRef<str>,
    ) -> EditResult {
        let line = self.get_or_add_line(y)?;
        line.insert_str(x, string.as_ref());

        Some((x + string.as_ref().n_chars(), y))
    }

    pub fn insert_strings_at(
        &mut self,
        (x, y): (usize, usize),
        strings: Vec<String>,
    ) -> EditResult {
        if strings.is_empty() {
            return Some((x, y));
        }

        let n_strings = strings.len();
        let last_line_len = self
            .lines
            .get(y + strings.len() - 1)
            .map(|s| s.len())
            .unwrap_or(0);

        // Handle the case where we're inserting in the middle of an existing line
        let is_middle_of_line = self.lines.get(y).is_some_and(|line| x < line.len());

        if is_middle_of_line {
            let line = self.lines.remove(y);
            let (left, right) = line.split_at(x);

            // Create the new first line
            let mut first_line = String::with_capacity(left.n_chars() + strings[0].n_chars());
            first_line.push_str(left);
            first_line.push_str(&strings[0]);

            // Build the remaining lines to insert
            let mut new_lines = Vec::with_capacity(strings.len());
            new_lines.push(first_line);
            new_lines.extend_from_slice(&strings[1..]);

            // Modify the last inserted line to include the right part
            let last_line_len = if let Some(last_line) = new_lines.last_mut() {
                last_line.push_str(right);
                last_line.n_chars()
            } else {
                0
            };

            self.lines.splice(
                y..=y,
                new_lines
                    .into_iter()
                    .map(DocumentLine::new)
                    .collect::<Vec<_>>(),
            );

            let final_y = y + strings.len() - 1;
            for line in &mut self.lines[final_y..] {
                line.is_dirty = true;
            }

            let final_x = last_line_len - right.n_chars();
            return Some((final_x, final_y));
        }

        // Simple case: insert at beginning of line or beyond existing content
        if y < self.lines.len() {
            self.lines
                .splice(y..y, strings.into_iter().map(DocumentLine::new));

            for line in &mut self.lines[y..] {
                line.is_dirty = true;
            }
        } else {
            self.lines
                .extend(strings.into_iter().map(DocumentLine::new));
        }

        Some((last_line_len, y + n_strings - 1))
    }

    pub fn insert_newline_at(&mut self, (x, y): (usize, usize)) -> EditResult {
        let line = self.get_or_add_line(y)?;

        let first_nonwhitespace_x = line
            .iter()
            .take(x)
            .position(|char| !char.is_whitespace())
            .unwrap_or(x);

        let mut to_append =
            DocumentLine::new(String::from(line.get_range(0..first_nonwhitespace_x)));

        if x < line.len() {
            to_append.push_str(line.split_off(x).as_ref())
        }

        self.lines.insert(y + 1, to_append);
		for line in &mut self.lines[y + 1..] {
			line.is_dirty = true;
		}

        Some((first_nonwhitespace_x, y + 1))
    }
}
