use crate::Document;
use crate::document::edit::{EditResult, Text};

impl Document {
    pub fn delete_char_at(&mut self, (x, y): (usize, usize)) -> (char, EditResult) {
        if x == 0 && (y == 0 || y >= self.lines.len()) {
            return ('0', None);
        }

        if x == 0 && y > 0 {
            // Merge with previous line
            let prev_line_len = self.lines.get(y - 1).map(|l| l.len()).unwrap_or_default();
            let current_line = self.lines.remove(y);
            self.lines[y - 1].push_str(current_line.as_ref());

            for line in &mut self.lines[y - 1..] {
                line.is_dirty = true;
            }

            return ('\n', Some((prev_line_len, y - 1)));
        }

        if let Some(line) = self.get_or_add_line(y) {
            let ch = line.remove(x - 1);
            if ch.is_none() {
                return ('0', None);
            }

            (ch.unwrap(), Some((x - 1, y)))
        } else {
            ('0', None)
        }
    }

    pub fn delete_range(&mut self, pos_from: (usize, usize), pos_to: (usize, usize)) -> Text {
        if pos_from.1 == pos_to.1
            && let Some(line) = self.lines.get_mut(pos_from.1)
        {
            let removed_text = line.remove_range(pos_from.0..pos_to.0);

            return Text::String(removed_text);
        }

        fn inner(
            doc: &mut Document,
            pos_from: (usize, usize),
            pos_to: (usize, usize),
        ) -> Option<Text> {
            let mut deleted_text = Vec::<String>::new();

            let to_line = doc.get_or_add_line(pos_to.1)?;
            let new_from_line_end = to_line.split_off(pos_to.0);

            let from_line = doc.get_or_add_line(pos_from.1)?;

            deleted_text.push(from_line.split_off(pos_from.0).into());
            from_line.push_str(new_from_line_end.as_ref());

            for line in doc.lines.drain((pos_from.1 + 1)..=pos_to.1) {
                deleted_text.push(line.into());
            }

            if pos_from.1 != pos_to.1 {
                for line in &mut doc.lines[pos_from.1..] {
                    line.is_dirty = true;
                }
            }

            Some(Text::Lines(deleted_text))
        }

        inner(self, pos_from, pos_to).unwrap_or_default()
    }
}
