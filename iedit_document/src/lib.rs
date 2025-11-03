mod edit;
mod io;
mod line;

use std::{
    fs::File,
    path::{Path, PathBuf},
};

pub use edit::{EditOperation, InverseStack, Text};
pub use line::{CharacterEditable, DocumentLine};
use regex_lite::Regex;

use crate::io::read_file;

pub struct Document {
    pub lines: Vec<String>,
    pub file: Option<File>,
    pub canonicalized_file_path: PathBuf,
    undo_stack: Vec<EditOperation>,
    redo_stack: Vec<EditOperation>,

    pub has_been_edited: bool,
    pub is_readonly: bool,
}

impl Document {
    pub fn from_lines(lines: Vec<String>, name: impl Into<PathBuf>, is_readonly: bool) -> Self {
        Self {
            lines,
            file: None,
            canonicalized_file_path: name.into(),
            undo_stack: vec![],
            redo_stack: vec![],
            has_been_edited: false,
            is_readonly,
        }
    }

    pub fn from_file(file_path: impl AsRef<Path>) -> std::io::Result<Self> {
        let (file, canonicalized_file_path, lines) = read_file(file_path)?;
        let is_readonly = if let Some(file) = &file
            && let Ok(metadata) = file.metadata()
        {
            metadata.permissions().readonly()
        } else {
            false
        };

        Ok(Self {
            lines,
            file,
            canonicalized_file_path,
            undo_stack: vec![],
            redo_stack: vec![],
            has_been_edited: false,
            is_readonly,
        })
    }

    pub fn get_name(&self) -> Option<&str> {
        None
    }

    #[inline]
    pub fn n_lines(&self) -> usize {
        self.lines.len()
    }

    #[inline]
    pub fn get_or_add_line(&mut self, y: usize) -> Option<&mut String> {
        if y < self.n_lines() {
            self.lines.get_mut(y)
        } else if y == self.n_lines() {
            self.lines.push(String::new());
            self.lines.last_mut()
        } else {
            None
        }
    }

    pub fn get_char_at_pos(&self, (x, y): (usize, usize)) -> Option<char> {
        self.lines.get(y).and_then(|line| line.get_nth_char(x))
    }

    pub fn get_next_word_pos(&self, (x, y): (usize, usize)) -> (usize, usize) {
        let line = match self.lines.get(y) {
            Some(line) => line,
            None => return (x, y),
        };

        let n_chars = line.n_chars();
        let mut next_word_x = x + 1;

        // Skip word
        while next_word_x <= n_chars
            && line
                .get_nth_char(next_word_x)
                .is_some_and(char::is_alphanumeric)
        {
            next_word_x += 1;
        }

        // Skip whitespace
        while next_word_x <= n_chars
            && line
                .get_nth_char(next_word_x)
                .is_some_and(char::is_whitespace)
        {
            next_word_x += 1;
        }

        (next_word_x, y)
    }

    pub fn get_previous_word_pos(&self, (x, y): (usize, usize)) -> (usize, usize) {
        let line = match self.lines.get(y) {
            Some(line) => line,
            None => return (0, y),
        };

        if x == 0 {
            return (x, y.saturating_sub(1));
        }

        let mut previous_word_x = x - 1;

        // Skip whitespace backwards
        while previous_word_x > 0
            && line
                .get_nth_char(previous_word_x)
                .is_some_and(char::is_whitespace)
        {
            previous_word_x -= 1;
        }

        // Skip word backwards
        while previous_word_x > 0
            && line
                .get_nth_char(previous_word_x)
                .is_some_and(char::is_alphanumeric)
        {
            previous_word_x -= 1;
        }

        (previous_word_x, y)
    }

    pub fn get_next_occurrence_of_char(
        &self,
        (x, y): (usize, usize),
        ch: char,
    ) -> Option<(usize, usize)> {
        // Search current line from position
        if let Some(line) = self.lines.get(y)
            && let Some(idx) = line[x..].find(ch)
        {
            return Some((x + idx, y));
        }

        // Search subsequent lines
        for (line_idx, line) in self.lines.iter().enumerate().skip(y + 1) {
            if let Some(idx) = line.find(ch) {
                return Some((idx, line_idx));
            }
        }

        None
    }

    pub fn get_word_boundaries(&self, (x, y): (usize, usize)) -> Option<(usize, usize)> {
        let line = self.lines.get(y)?;
        if line.get_nth_char(x).is_none_or(char::is_whitespace) {
            return None;
        };

        let left_boundary = line
            .get_chars(0..(x + 1))
            .char_indices()
            .rfind(|(_idx, ch)| ch.is_whitespace())
            .map(|(idx, _ch)| idx + 1)
            .unwrap_or_default();

        let right_boundary = line
            .get_chars(x..)
            .char_indices()
            .find(|(_idx, ch)| ch.is_whitespace())
            .map(|(idx, _ch)| idx.saturating_sub(1))
            .unwrap_or(line.n_chars() - x);

        Some((left_boundary, right_boundary + x))
    }

    pub fn get_previous_occurrence_of_char(
        &self,
        (x, y): (usize, usize),
        ch: char,
    ) -> Option<(usize, usize)> {
        // Search current line backwards from position
        if let Some(line) = self.lines.get(y)
            && let Some(idx) = line[..x].rfind(ch)
        {
            return Some((idx, y));
        }

        // Search previous lines backwards
        for line_idx in (0..y).rev() {
            if let Some(line) = self.lines.get(line_idx)
                && let Some(idx) = line.rfind(ch)
            {
                return Some((idx, line_idx));
            }
        }

        None
    }

    pub fn get_next_blank_line_idx(&self, pos_y: usize) -> usize {
        for (idx, line) in self.lines.iter().enumerate().skip(pos_y) {
            if line.trim().is_empty() {
                return idx;
            }
        }
        self.lines.len()
    }

    pub fn get_previous_blank_line_idx(&self, pos_y: usize) -> usize {
        for idx in (0..pos_y).rev() {
            if self.lines.get(idx).is_some_and(|line| line.is_empty()) {
                return idx;
            }
        }
        0
    }

    pub fn get_next_literal_match_pos(
        &self,
        from_pos: (usize, usize),
        lit: &str,
    ) -> Option<(usize, usize)> {
        for y in from_pos.1..self.n_lines() {
            let line = if y == from_pos.1 {
                self.lines
                    .get(y)
                    .map(|line| line.get_chars(from_pos.0 + 1..))
            } else {
                self.lines.get(y).map(|line| line.as_str())
            }?;

            let offset = from_pos.0 * (y == from_pos.1) as usize;
            if let Some(idx) = line.find(lit) {
                let x = line.char_idx_at_byte(idx)?;
                return Some((x + offset, y));
            }
        }

        None
    }

    pub fn get_next_regex_match_pos(
        &self,
        from_pos: (usize, usize),
        regex: &Regex,
    ) -> Option<(usize, usize)> {
        for y in from_pos.1..self.n_lines() {
            let line = if y == from_pos.1 {
                self.lines
                    .get(y)
                    .map(|line| line.get_chars(from_pos.0 + 1..))
            } else {
                self.lines.get(y).map(|line| line.as_str())
            }?;

            let offset = from_pos.0 * (y == from_pos.1) as usize;
            if let Some(reg_match) = regex.find(line) {
                let x = line.char_idx_at_byte(reg_match.start())?;
                return Some((x + offset, y));
            }
        }

        None
    }

    pub fn get_previous_literal_match_pos(
        &self,
        from_pos: (usize, usize),
        lit: &str,
    ) -> Option<(usize, usize)> {
        for y in (0..=from_pos.1).rev() {
            let line = if y == from_pos.1 {
                self.lines.get(y).map(|line| line.get_chars(..from_pos.0))
            } else {
                self.lines.get(y).map(|line| line.as_str())
            }?;

            if let Some(idx) = line.rfind(lit) {
                let x = line.char_idx_at_byte(idx)?;
                return Some((x, y));
            }
        }

        None
    }

    pub fn get_previous_regex_match_pos(
        &self,
        from_pos: (usize, usize),
        regex: &Regex,
    ) -> Option<(usize, usize)> {
        for y in (0..=from_pos.1).rev() {
            let line = if y == from_pos.1 {
                self.lines.get(y).map(|line| line.get_chars(..from_pos.0))
            } else {
                self.lines.get(y).map(|line| line.as_str())
            }?;

            if let Some(reg_match) = regex.find_iter(line).last() {
                let x = line.char_idx_at_byte(reg_match.start())?;
                return Some((x, y));
            }
        }

        None
    }

    pub fn get_matching_paren_pos(&self, (x, y): (usize, usize)) -> Option<(usize, usize)> {
        let line = self.lines.get(y)?;
        let start_char = line.get_nth_char(x)?;

        let (matching_char, direction) = match start_char {
            '(' => (')', 1),
            ')' => ('(', -1),
            '[' => (']', 1),
            ']' => ('[', -1),
            '{' => ('}', 1),
            '}' => ('{', -1),
            _ => return None,
        };

        let mut stack = 0;

        if direction == 1 {
            // Search forward
            for (line_idx, current_line) in self.lines.iter().enumerate().skip(y) {
                let start_pos = if line_idx == y { x + 1 } else { 0 };
                for (char_idx, ch) in current_line.iter_chars().enumerate().skip(start_pos) {
                    if ch == start_char {
                        stack += 1;
                    } else if ch == matching_char {
                        if stack == 0 {
                            return Some((char_idx, line_idx));
                        }
                        stack -= 1;
                    }
                }
            }
        } else {
            // Search backward
            for line_idx in (0..=y).rev() {
                let current_line = &self.lines[line_idx];
                let end_pos = if line_idx == y {
                    x
                } else {
                    current_line.n_chars()
                };
                for char_idx in (0..end_pos).rev() {
                    let ch = current_line.get_nth_char(char_idx).unwrap();
                    if ch == start_char {
                        stack += 1;
                    } else if ch == matching_char {
                        if stack == 0 {
                            return Some((char_idx, line_idx));
                        }
                        stack -= 1;
                    }
                }
            }
        }

        None
    }
}
