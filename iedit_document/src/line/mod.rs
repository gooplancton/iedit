mod char_indexable;

pub use char_indexable::CharacterIndexable;
use std::ops::RangeBounds;

/// A line in a document. This is mostly a wrapper around String with some additional metadata.
/// All indexing operations are done in terms of characters, not bytes.
pub struct DocumentLine {
    buf: String,
    /// NOTE: every line is considered dirty when created or modified.
    pub needs_render: bool,
    pub has_been_modified: bool,
}

impl Default for DocumentLine {
    fn default() -> Self {
        DocumentLine {
            buf: String::new(),
            needs_render: true,
            has_been_modified: true,
        }
    }
}

impl DocumentLine {
    pub fn new(line: String) -> Self {
        DocumentLine {
            buf: line,
            needs_render: true,
            has_been_modified: true,
        }
    }

    #[inline(always)]
    pub fn set_dirty(&mut self) {
        self.has_been_modified = true;
        self.needs_render = true;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.buf.chars().count()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    #[inline(always)]
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.buf.starts_with(prefix)
    }

    #[inline(always)]
    pub fn push(&mut self, ch: char) {
        self.buf.push(ch);
        self.set_dirty();
    }

    #[inline(always)]
    pub fn push_str(&mut self, string: &str) {
        self.buf.push_str(string);
        self.set_dirty();
    }

    #[inline(always)]
    pub fn truncate(&mut self, new_len: usize) {
        let byte_idx = self.char_to_byte_idx(new_len.saturating_sub(1));
        self.buf.truncate(byte_idx);
        self.set_dirty();
    }

    #[inline(always)]
    pub fn insert(&mut self, idx: usize, ch: char) {
        let byte_idx = self.char_to_byte_idx(idx);
        self.buf.insert(byte_idx, ch);
        self.set_dirty();
    }

    #[inline(always)]
    pub fn insert_str(&mut self, idx: usize, string: &str) {
        let byte_idx = self.char_to_byte_idx(idx);
        self.buf.insert_str(byte_idx, string);
        self.set_dirty();
    }

    #[inline(always)]
    pub fn at(&self, idx: usize) -> Option<char> {
        self.buf.chars().nth(idx)
    }

    pub fn remove(&mut self, idx: usize) -> Option<char> {
        let byte_idx = self.char_to_byte_idx(idx);
        let ch = self.buf[byte_idx..].chars().next()?;
        let ch_len = ch.len_utf8();
        self.buf.replace_range(byte_idx..byte_idx + ch_len, "");
        self.set_dirty();

        Some(ch)
    }

    #[inline]
    pub fn remove_range(&mut self, range: impl RangeBounds<usize>) -> String {
        let byte_range = self.char_to_byte_range(range);
        let removed = self.buf[byte_range.clone()].to_owned();
        self.buf.replace_range(byte_range, "");
        self.set_dirty();

        removed
    }

    #[inline]
    pub fn get_range(&self, range: impl RangeBounds<usize>) -> &str {
        let byte_range = self.char_to_byte_range(range);
        &self.buf[byte_range]
    }

    #[inline]
    pub fn get_range_mut(&mut self, range: impl RangeBounds<usize>) -> &mut str {
        let byte_range = self.char_to_byte_range(range);
        &mut self.buf[byte_range]
    }

    #[inline]
    pub fn char_to_byte_idx(&self, char_idx: usize) -> usize {
        self.buf
            .char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(self.buf.len())
    }

    #[inline]
    pub fn byte_to_char_idx(&self, byte_idx: usize) -> Option<usize> {
        self.buf
            .char_indices()
            .enumerate()
            .find(|(_, (char_byte_idx, _))| *char_byte_idx == byte_idx)
            .map(|(char_idx, _)| char_idx)
    }

    #[inline]
    pub fn char_to_visual_idx(&self, char_idx: usize, tab_size: usize) -> usize {
        let mut visual_idx = 0;
        for ch in self.iter().take(char_idx) {
            if ch != '\t' {
                visual_idx += 1;
            } else {
                visual_idx += tab_size - (visual_idx % tab_size)
            }
        }

        visual_idx
    }

    #[inline]
    pub fn visual_to_char_idx(&self, visual_idx: usize, tab_size: usize) -> usize {
        let mut running_visual_idx = 0;
        for (char_idx, ch) in self.iter().enumerate() {
            running_visual_idx += if ch == '\t' {
                tab_size - (running_visual_idx % tab_size)
            } else {
                1
            };

            if running_visual_idx > visual_idx {
                return char_idx;
            }
        }

        self.len()
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = char> {
        self.buf.chars()
    }

    #[inline]
    pub fn split_off(&mut self, idx: usize) -> Self {
        let byte_idx = self.char_to_byte_idx(idx);
        let other_buf = self.buf.split_off(byte_idx);
        self.set_dirty();
        DocumentLine::new(other_buf)
    }

    #[inline]
    pub fn split_at(&self, idx: usize) -> (&str, &str) {
        let byte_idx = self.char_to_byte_idx(idx);

        self.buf.split_at(byte_idx)
    }

    #[inline]
    pub fn split_at_mut(&mut self, idx: usize) -> (&mut str, &mut str) {
        let byte_idx = self.char_to_byte_idx(idx);

        self.buf.split_at_mut(byte_idx)
    }

    fn char_to_byte_range(&self, range: impl RangeBounds<usize>) -> std::ops::Range<usize> {
        let start = match range.start_bound() {
            std::ops::Bound::Included(&idx) => self.char_to_byte_idx(idx),
            std::ops::Bound::Excluded(&idx) => self.char_to_byte_idx(idx + 1),
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(&idx) => self.char_to_byte_idx(idx + 1),
            std::ops::Bound::Excluded(&idx) => self.char_to_byte_idx(idx),
            std::ops::Bound::Unbounded => self.buf.len(),
        };
        start..end
    }
}

impl AsRef<str> for DocumentLine {
    fn as_ref(&self) -> &str {
        &self.buf
    }
}

impl Into<String> for DocumentLine {
    fn into(self) -> String {
        self.buf
    }
}
