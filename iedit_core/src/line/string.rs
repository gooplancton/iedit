use crate::line::{CharacterEditable, EditorLine};

impl CharacterEditable for str {
    fn len(&self) -> usize {
        self.chars().count()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn iter_chars(&self) -> impl Iterator<Item = char> {
        self.chars()
    }

    fn split_chars_at(&self, idx: usize) -> (&Self, &Self) {
        let char_idx = self
            .char_indices()
            .nth(idx)
            .map(|(i, _)| i)
            .unwrap_or(self.len());

        self.split_at(char_idx)
    }

    fn split_chars_at_mut(&mut self, idx: usize) -> (&mut Self, &mut Self) {
        let char_idx = self
            .char_indices()
            .nth(idx)
            .map(|(i, _)| i)
            .unwrap_or(self.len());

        self.split_at_mut(char_idx)
    }

    fn to_string(&self) -> String {
        self.to_owned()
    }
}

impl EditorLine for String {
    type SliceType = str;

    fn new() -> Self {
        String::new()
    }

    fn len(&self) -> usize {
        self.chars().count()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn as_str(&self) -> &str {
        self.as_str()
    }

    fn from_str(string: &impl AsRef<str>) -> Self {
        string.as_ref().trim_end_matches("\n").to_owned()
    }

    fn to_string(&self) -> String {
        self.clone()
    }

    fn iter_chars(&self) -> impl Iterator<Item = char> {
        self.chars()
    }

    fn merge_at_end(&mut self, other: &mut Self) {
        self.push_str(other.as_str());
    }

    fn split_chars_off_at(&mut self, idx: usize) -> Self {
        let char_idx = self
            .char_indices()
            .nth(idx)
            .map(|(i, _)| i)
            .unwrap_or(self.len());
        self.split_off(char_idx)
    }

    fn delete_chars(&mut self, range: impl std::ops::RangeBounds<usize>) {
        let start = match range.start_bound() {
            std::ops::Bound::Included(&idx) => self
                .char_indices()
                .nth(idx)
                .map(|(i, _)| i)
                .unwrap_or(self.len()),
            std::ops::Bound::Excluded(&idx) => self
                .char_indices()
                .nth(idx + 1)
                .map(|(i, _)| i)
                .unwrap_or(self.len()),
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(&idx) => self
                .char_indices()
                .nth(idx + 1)
                .map(|(i, _)| i)
                .unwrap_or(self.len()),
            std::ops::Bound::Excluded(&idx) => self
                .char_indices()
                .nth(idx)
                .map(|(i, _)| i)
                .unwrap_or(self.len()),
            std::ops::Bound::Unbounded => self.len(),
        };
        if start < end && end <= self.len() {
            self.replace_range(start..end, "");
        }
    }

    fn push_char(&mut self, ch: char) {
        self.push(ch)
    }

    fn insert_char_at(&mut self, ch: char, idx: usize) {
        let char_idx = self
            .char_indices()
            .nth(idx)
            .map(|(i, _)| i)
            .unwrap_or(self.len());
        self.insert(char_idx, ch);
    }

    fn remove_char_at(&mut self, idx: usize) -> char {
        let char_idx = self
            .char_indices()
            .nth(idx)
            .map(|(i, _)| i)
            .unwrap_or(self.len());
        if char_idx < self.len() {
            let ch = self[char_idx..].chars().next().unwrap();
            self.replace_range(char_idx..char_idx + ch.len_utf8(), "");
            ch
        } else {
            panic!("Index out of bounds");
        }
    }

    fn get_chars(&self, range: std::ops::Range<usize>) -> &Self::SliceType {
        let start = self
            .char_indices()
            .nth(range.start)
            .map(|(i, _)| i)
            .unwrap_or(self.len());
        let end = self
            .char_indices()
            .nth(range.end)
            .map(|(i, ch)| i + ch.len_utf8() - 1)
            .unwrap_or(self.len());
        &self[start..end]
    }

    fn get_nth_char(&self, idx: usize) -> Option<char> {
        self.chars().nth(idx)
    }

    fn push_str(&mut self, string: &str) {
        self.push_str(string);
    }

    fn truncate_chars(&mut self, new_len: usize) {
        if new_len == 0 {
            self.truncate(0);
        } else {
            let char_idx = self
                .char_indices()
                .nth(new_len - 1)
                .map(|(i, _)| i)
                .unwrap_or(self.len());
            self.truncate(char_idx);
        }
    }

    fn find_term(&self, term: &str) -> Option<usize> {
        self.find(term)
    }
}
