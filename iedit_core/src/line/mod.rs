mod char_vec;
mod string;
mod highlight;
mod renderer;

pub use highlight::SelectionHighlight;
pub use renderer::LineRenderer;
use std::ops::{Range, RangeBounds};

pub trait CharacterEditable {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn iter_chars(&self) -> impl Iterator<Item = char>;
    fn split_chars_at(&self, idx: usize) -> (&Self, &Self);
    fn split_chars_at_mut(&mut self, idx: usize) -> (&mut Self, &mut Self);
    //fn multi_split_chars_at(&self, idxs: &[usize]) -> Vec<&Self>;
    //fn multi_split_chars_at_mut(&mut self, idxs: &[usize]) -> Vec<&mut Self>;
    fn to_string(&self) -> String;
}

pub trait EditorLine: Default {
    type SliceType: ?Sized + CharacterEditable;

    fn new() -> Self;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn from_str(string: &impl AsRef<str>) -> Self;
    fn as_str(&self) -> &str;
    fn to_string(&self) -> String;
    fn iter_chars(&self) -> impl Iterator<Item = char>;
    fn merge_at_end(&mut self, other: &mut Self);
    fn split_chars_off_at(&mut self, idx: usize) -> Self;
    fn delete_chars(&mut self, range: impl RangeBounds<usize>);
    fn push_char(&mut self, ch: char);
    fn push_str(&mut self, string: &str);
    fn insert_char_at(&mut self, ch: char, idx: usize);
    fn remove_char_at(&mut self, idx: usize) -> char;
    fn get_chars(&self, range: Range<usize>) -> &Self::SliceType;
    fn get_nth_char(&self, idx: usize) -> Option<char>;
    fn truncate_chars(&mut self, new_len: usize);
}

pub type DefaultLineType = String;
