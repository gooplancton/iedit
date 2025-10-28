use std::ops::RangeBounds;

use regex_lite::Regex;

use crate::line::{CharacterEditable, DocumentLine};

impl CharacterEditable for str {
    fn n_chars(&self) -> usize {
        self.chars().count()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn iter_chars(&self) -> impl Iterator<Item = char> {
        self.chars()
    }

    fn nth_char_idx(&self, n: usize) -> usize {
        self.char_indices()
            .nth(n)
            .map(|(i, _)| i)
            .unwrap_or(self.len())
    }

    fn char_idx_at_byte(&self, byte_idx: usize) -> Option<usize> {
        self.char_indices()
            .enumerate()
            .find(|(_, (char_byte_idx, _))| *char_byte_idx == byte_idx)
            .map(|(char_idx, _)| char_idx)
    }

    fn split_chars_at(&self, idx: usize) -> (&Self, &Self) {
        let char_idx = self.nth_char_idx(idx);

        self.split_at(char_idx)
    }

    fn split_chars_at_mut(&mut self, idx: usize) -> (&mut Self, &mut Self) {
        let char_idx = self.nth_char_idx(idx);

        self.split_at_mut(char_idx)
    }

    fn as_str(&self) -> &str {
        self
    }

    fn to_string(&self) -> String {
        self.to_owned()
    }

    fn find_regex_from(&self, regex: &Regex, idx: usize) -> Option<(usize, usize)> {
        let char_idx = self.nth_char_idx(idx);

        regex.find_at(self, char_idx).map(|m| {
            let match_range = m.range();
            let (start, end) = (match_range.start, match_range.end);

            (
                self.char_idx_at_byte(start).unwrap_or(start),
                self.char_idx_at_byte(end).unwrap_or(end),
            )
        })
    }

    fn find_last_match(&self, regex: &Regex) -> Option<(usize, usize)> {
        let (mut start_idx, mut end_idx) = self.find_regex_from(regex, 0)?;

        loop {
            let from_idx = self.nth_char_idx(start_idx + 1);

            if let Some((last_start_idx, last_end_idx)) = self.find_regex_from(regex, from_idx) {
                start_idx = last_start_idx;
                end_idx = last_end_idx;
            } else {
                return Some((start_idx, end_idx));
            }
        }
    }
}

impl DocumentLine for String {
    fn n_chars(&self) -> usize {
        self.chars().count()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn as_str(&self) -> &str {
        self.as_str()
    }

    fn from_str_trim_newline(string: &impl AsRef<str>) -> Self {
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

    fn nth_char_idx(&self, n: usize) -> usize {
        self.char_indices()
            .nth(n)
            .map(|(i, _)| i)
            .unwrap_or(self.len())
    }

    fn char_idx_at_byte(&self, byte_idx: usize) -> Option<usize> {
        self.char_indices()
            .find(|(char_idx, _)| *char_idx == byte_idx)
            .map(|(char_idx, _)| char_idx)
    }

    fn split_chars_off_at(&mut self, idx: usize) -> Self {
        let char_idx = self
            .char_indices()
            .nth(idx)
            .map(|(i, _)| i)
            .unwrap_or(self.len());
        self.split_off(char_idx)
    }

    fn delete_chars(&mut self, range: impl std::ops::RangeBounds<usize>) -> Option<String> {
        let start = match range.start_bound() {
            std::ops::Bound::Included(&idx) => self.nth_char_idx(idx),
            std::ops::Bound::Excluded(&idx) => self.nth_char_idx(idx + 1),
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(&idx) => self.nth_char_idx(idx + 1),
            std::ops::Bound::Excluded(&idx) => self.nth_char_idx(idx),
            std::ops::Bound::Unbounded => self.len(),
        };
        if start < end && end <= self.n_chars() {
            let mut rest = self.split_off(start);
            let mut to_attach = rest.split_off(end - start);
            self.merge_at_end(&mut to_attach);

            return Some(rest);
        }

        None
    }

    fn push_char(&mut self, ch: char) {
        self.push(ch)
    }

    fn insert_char_at(&mut self, ch: char, idx: usize) {
        let char_idx = self.nth_char_idx(idx);

        self.insert(char_idx, ch);
    }

    fn remove_char_at(&mut self, idx: usize) -> Option<char> {
        let char_idx = self.nth_char_idx(idx);

        if char_idx <= self.len() {
            let ch = self[char_idx..].chars().next()?;
            self.replace_range(char_idx..char_idx + ch.len_utf8(), "");
            Some(ch)
        } else {
            None
        }
    }

    fn get_chars(&self, range: impl RangeBounds<usize>) -> &str {
        let start = match range.start_bound() {
            std::ops::Bound::Included(&idx) => self.nth_char_idx(idx),
            std::ops::Bound::Excluded(&idx) => self.nth_char_idx(idx + 1),
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(&idx) => self.nth_char_idx(idx + 1),
            std::ops::Bound::Excluded(&idx) => self.nth_char_idx(idx),
            std::ops::Bound::Unbounded => self.len(),
        };
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
            let char_idx = self.nth_char_idx(new_len - 1);

            self.truncate(char_idx);
        }
    }

    fn insert_str_at(&mut self, idx: usize, string: &str) {
        let char_idx = self.nth_char_idx(idx);

        self.insert_str(char_idx, string);
    }

    fn find_regex_from(&self, regex: &Regex, idx: usize) -> Option<(usize, usize)> {
        let char_idx = self.nth_char_idx(idx);

        regex.find_at(self, char_idx).map(|m| {
            let match_range = m.range();
            let (start, end) = (match_range.start, match_range.end);

            (
                self.char_idx_at_byte(start).unwrap_or(start),
                self.char_idx_at_byte(end).unwrap_or(end),
            )
        })
    }
}
