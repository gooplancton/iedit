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
        } else if strings.len() == 1 {
            let line = self.get_or_add_line(y)?;
            line.insert_str(x, &strings[0]);
            return Some((x + strings[0].n_chars(), y));
        } else if y >= self.n_lines() {
            self.lines
                .extend(strings.into_iter().map(DocumentLine::new));
            return Some((0, self.n_lines()));
        }

        let first_line = &mut self.lines[y];
        let last_line_postfix = first_line.split_off(x);
        let landing_x = x + strings.last().unwrap().n_chars();
        let landing_y = y + strings.len() - 1;
        first_line.push_str(&strings[0]);
        let n_strings = strings.len();
        for (idx, string) in strings.into_iter().enumerate().skip(1) {
            let line = if idx == n_strings - 1 {
                let mut line = DocumentLine::new(string);
                line.push_str(last_line_postfix.as_ref());
                
                line
            } else {
                DocumentLine::new(string)
            };

            self.lines.insert(y + idx, line);
        }

        for line in &mut self.lines[y..] {
            line.needs_render = true;
        }

        Some((landing_x, landing_y))
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
            line.needs_render = true;
        }

        Some((first_nonwhitespace_x, y + 1))
    }
}
