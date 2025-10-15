use core::slice;
use std::ops::{Range, RangeBounds};

use crate::line::{CharacterEditable, EditorLine};

impl EditorLine for Vec<char> {
    type SliceType = [char];

    fn new() -> Self {
        Self::new()
    }

    fn from_str(string: &impl AsRef<str>) -> Self {
        string.as_ref().trim_end_matches("\n").chars().collect()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn to_string(&self) -> String {
        self.iter().collect()
    }

    fn iter_chars(&self) -> impl Iterator<Item = &char> {
        self.iter()
    }

    fn split_chars_off_at(&mut self, idx: usize) -> Self {
        self.split_off(idx)
    }

    fn merge_at_end(&mut self, other: &mut Self) {
        self.append(other);
    }

    fn delete_chars(&mut self, range: impl RangeBounds<usize>) {
        self.drain(range);
    }

    fn push_char(&mut self, ch: char) {
        self.push(ch)
    }

    fn insert_char_at(&mut self, ch: char, idx: usize) {
        self.insert(idx, ch);
    }

    fn remove_char_at(&mut self, idx: usize) -> char {
        self.remove(idx)
    }

    fn get_chars(&self, range: Range<usize>) -> &Self::SliceType {
        self.get(range).unwrap_or_default()
    }

    fn get_nth_char(&self, idx: usize) -> Option<&char> {
        self.get(idx)
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl CharacterEditable for [char] {
    fn len(&self) -> usize {
        self.len()
    }

    fn iter_chars(&self) -> impl Iterator<Item = &char> {
        self.iter()
    }

    fn split_chars_at(&self, idx: usize) -> (&Self, &Self) {
        self.split_at(idx)
    }

    fn split_chars_at_mut(&mut self, idx: usize) -> (&mut Self, &mut Self) {
        self.split_at_mut(idx)
    }

    //fn multi_split_chars_at(&self, idxs: &[usize]) -> Vec<&[char]> {
    //    let n_chunks = idxs.len() + 1;
    //    if n_chunks == 1 {
    //        return Vec::from([self]);
    //    }
    //
    //    if !idxs.is_sorted() || *idxs.last().unwrap() > self.len() {
    //        panic!("invalid indices {:?} for {}", idxs, self.to_string())
    //    }
    //
    //    let mut res = Vec::with_capacity(n_chunks);
    //    let mut to_split = self;
    //    let mut offset = 0;
    //    for idx in 0..(n_chunks - 1) {
    //        let (chunk, rest) = to_split.split_at(idx + offset);
    //        offset += chunk.len();
    //        res.push(chunk);
    //        if idx == n_chunks - 2 {
    //            res.push(rest);
    //        } else {
    //            to_split = rest;
    //        }
    //    }
    //
    //    res
    //}
    //
    //fn multi_split_chars_at_mut(&mut self, idxs: &[usize]) -> Vec<&mut [char]> {
    //    let n_chunks = idxs.len() + 1;
    //    if n_chunks == 1 {
    //        return Vec::from([self]);
    //    }
    //
    //    if !idxs.is_sorted() || *idxs.last().unwrap() > self.len() {
    //        panic!("invalid indices {:?} for {}", idxs, self.to_string())
    //    }
    //
    //    let mut res = Vec::with_capacity(n_chunks);
    //    let mut offset = 0;
    //    for idx in 0..(n_chunks - 1) {
    //        let to_split_len = self.len() - offset;
    //        let to_split = unsafe {
    //            let to_split_start = self.as_mut_ptr().add(offset);
    //            slice::from_raw_parts_mut(to_split_start, to_split_len)
    //        };
    //
    //        let (chunk, rest) = to_split.split_at_mut(idx);
    //        offset += chunk.len();
    //        res.push(chunk);
    //        if idx == n_chunks - 2 {
    //            res.push(rest);
    //        }
    //    }
    //
    //    res
    //}

    fn to_string(&self) -> String {
        self.iter().collect()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}
