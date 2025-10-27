use crate::{Document, DocumentLine};

impl Document {
    pub fn insert_char_at(&mut self, (x, y): (usize, usize), ch: char) {
        let line = self.get_or_add_line(y);
        line.insert_char_at(ch, x);
    }

    pub fn insert_string_at(&mut self, (x, y): (usize, usize), string: impl AsRef<str>) {
        let line = self.get_or_add_line(y);
        line.insert_str_at(x, string.as_ref());
    }
}

