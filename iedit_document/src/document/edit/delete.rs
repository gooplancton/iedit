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

    /// NOTE: pos_to is EXCLUSIVE in the sense that:
    /// - (0, y) (self.lines\[y\].len(), y) will delete all the characters in the line and leave an empty line in its place
    /// and return a Text::String with the contents of the deleted line
    /// - (0, y) (0, y + 1) will delete line y and shift all lines up by 1
    /// and return a Text::Lines with a vec like \[ self.lines\[y\], "" \]
    pub fn delete_range(&mut self, pos_from: (usize, usize), pos_to: (usize, usize)) -> Text {
        if pos_from.1 == pos_to.1
            && let Some(line) = self.lines.get_mut(pos_from.1)
        {
            let removed_text = line.remove_range(pos_from.0..pos_to.0);

            return Text::String(removed_text);
        }

        let mut deleted_text = Vec::<String>::new();

        // splice first line: doc.lines[pos_from.1][..pos_from.0] + doc.lines[pos_to.1][pos_to.0..]
        let deleted_last = {
            let last_line = if pos_to.1 < self.n_lines() {
                Some(self.lines.remove(pos_to.1))
            } else {
                None
            };

            let first_line = &mut self.lines[pos_from.1];

            deleted_text.push(first_line.split_off(pos_from.0).into());

            if let Some(mut last_line) = last_line {
                let to_append = last_line.split_off(pos_to.0);
                first_line.push_str(to_append.as_ref());
                Some(last_line)
            } else {
                None
            }
        };

        for line in self.lines.drain((pos_from.1 + 1)..pos_to.1) {
            deleted_text.push(line.into());
        }

        if let Some(deleted_last) = deleted_last {
            deleted_text.push(deleted_last.into());
        }

        for line in &mut self.lines[pos_from.1..] {
            line.is_dirty = true;
        }

        Text::Lines(deleted_text)
    }
}
