use crate::{Document, DocumentLine, Text, edit::EditResult};

impl Document {
    pub fn delete_char_at(&mut self, (x, y): (usize, usize)) -> EditResult {
        if x == 0 && y == 0 {
            return None;
        }

        if x == 0 && y > 0 {
            // Merge with previous line
            let prev_line_len = self.lines.get(y - 1).map(|l| l.len()).unwrap_or_default();
            let mut current_line = self.lines.remove(y);
            self.lines[y - 1].merge_at_end(&mut current_line);

            return Some((prev_line_len, y - 1));
        }

        let line = self.get_or_add_line(y)?;
        line.remove_char_at(x - 1);

        Some((x - 1, y))
    }

    pub fn delete_range(&mut self, pos_from: (usize, usize), pos_to: (usize, usize)) -> Text {
        if pos_from.1 == pos_to.1
            && let Some(line) = self.lines.get_mut(pos_from.1)
        {
            return match line.delete_chars(pos_from.0..pos_to.1) {
                Some(string) => Text::String(string),
                None => Text::Empty,
            };
        };

        fn inner(
            doc: &mut Document,
            pos_from: (usize, usize),
            pos_to: (usize, usize),
        ) -> Option<Text> {
            let mut deleted_text = Vec::<String>::new();

            let to_line = doc.get_or_add_line(pos_to.1)?;
            let mut new_from_line_end = to_line.split_chars_off_at(pos_to.0);

            let from_line = doc.get_or_add_line(pos_from.1)?;

            deleted_text.push(from_line.split_chars_off_at(pos_from.0));
            from_line.merge_at_end(&mut new_from_line_end);

            for line in doc.lines.drain((pos_from.1 + 1)..pos_to.1) {
                deleted_text.push(line);
            }

            Some(Text::Lines(deleted_text))
        }

        inner(self, pos_from, pos_to).unwrap_or_default()
    }
}
