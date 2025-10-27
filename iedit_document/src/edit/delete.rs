use crate::{Document, DocumentLine};

impl Document {
    pub fn delete_char_at(&mut self, (x, y): (usize, usize)) {
        let line = self.get_or_add_line(y);
        line.delete_chars(x..(x + 1));
    }

    pub fn delete_range(&mut self, (from_x, from_y): (usize, usize), (to_x, to_y): (usize, usize)) {
        let to_line = self.get_or_add_line(to_y);
        let mut to_attach = to_line.split_chars_off_at(to_x);
        let from_line = self.get_or_add_line(from_y);
        from_line.split_chars_off_at(from_x);
        from_line.merge_at_end(&mut to_attach);
        self.lines.drain((from_y+1)..to_y);
    }
}
