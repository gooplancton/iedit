mod string;

use std::ops::RangeBounds;

pub trait CharacterEditable {
    fn n_chars(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn nth_char_idx(&self, idx: usize) -> usize;
    fn char_idx_at_byte(&self, byte_idx: usize) -> Option<usize>;
    fn iter_chars(&self) -> impl Iterator<Item = char>;
    fn split_chars_at(&self, idx: usize) -> (&Self, &Self);
    fn split_chars_at_mut(&mut self, idx: usize) -> (&mut Self, &mut Self);
    fn as_str(&self) -> &str;
    fn to_string(&self) -> String;
}

pub trait DocumentLine: Default {
    fn n_chars(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn nth_char_idx(&self, idx: usize) -> usize;
    fn char_idx_at_byte(&self, byte_idx: usize) -> Option<usize>;
    fn from_str_trim_newline(string: &impl AsRef<str>) -> Self;
    fn as_str(&self) -> &str;
    fn to_string(&self) -> String;
    fn iter_chars(&self) -> impl Iterator<Item = char>;
    fn merge_at_end(&mut self, other: &mut Self);
    fn split_chars_off_at(&mut self, idx: usize) -> Self;
    fn delete_chars(&mut self, range: impl RangeBounds<usize>) -> Option<String>;
    fn push_char(&mut self, ch: char);
    fn push_str(&mut self, string: &str);
    fn insert_str_at(&mut self, idx: usize, string: &str);
    fn insert_char_at(&mut self, ch: char, idx: usize);
    fn remove_char_at(&mut self, idx: usize) -> char;
    fn get_chars(&self, range: impl RangeBounds<usize>) -> &str;
    fn get_nth_char(&self, idx: usize) -> Option<char>;
    fn truncate_chars(&mut self, new_len: usize);
}
