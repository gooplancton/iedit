mod edit;
mod line;

pub use edit::{EditOperation, Text};
pub use line::{CharacterEditable, DocumentLine};

pub struct Document {
    pub lines: Vec<String>,
    // undo_stack: Vec<EditOperation>,
    // redo_stack: Vec<EditOperation>,
    pub has_been_edited: bool,
}

impl Document {
    pub fn new(lines: Vec<String>) -> Self {
        Self {
            lines,
            // undo_stack: vec![],
            // redo_stack: vec![],
            has_been_edited: false,
        }
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

    // TODO: review AI boilerplate

    pub fn get_next_word_x(&self, (x, y): (usize, usize)) -> usize {
        let line = match self.lines.get(y) {
            Some(line) => line,
            None => return x,
        };

        let chars: Vec<char> = line.chars().collect();

        // If we're already beyond the line, stay there
        if x >= chars.len() {
            return chars.len();
        }

        // Skip current word if we're in the middle of one
        let mut pos = x;
        while pos < chars.len() && chars[pos].is_alphanumeric() {
            pos += 1;
        }

        // Skip whitespace
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }

        pos
    }

    pub fn get_previous_word_x(&self, (x, y): (usize, usize)) -> usize {
        let line = match self.lines.get(y) {
            Some(line) => line,
            None => return 0,
        };

        let chars: Vec<char> = line.chars().collect();

        // If at start, stay there
        if x == 0 {
            return 0;
        }

        let mut pos = x.min(chars.len());

        // Skip whitespace backwards
        while pos > 0 && chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        // Skip word backwards
        while pos > 0 && chars[pos - 1].is_alphanumeric() {
            pos -= 1;
        }

        pos
    }

    pub fn get_word_start_x(&self, (x, y): (usize, usize)) -> usize {
        let line = match self.lines.get(y) {
            Some(line) => line,
            None => return 0,
        };

        let chars: Vec<char> = line.chars().collect();
        let mut pos = x.min(chars.len());

        // If we're on whitespace or at line end, find previous word start
        if pos == chars.len() || chars[pos].is_whitespace() {
            return self.get_previous_word_x((pos, y));
        }

        // Move backwards to word start
        while pos > 0 && chars[pos - 1].is_alphanumeric() {
            pos -= 1;
        }

        pos
    }

    pub fn get_next_occurrence_of_char(
        &self,
        (x, y): (usize, usize),
        ch: char,
    ) -> Option<(usize, usize)> {
        // Search current line from position
        if let Some(line) = self.lines.get(y) {
            if let Some(idx) = line[x..].find(ch) {
                return Some((x + idx, y));
            }
        }

        // Search subsequent lines
        for (line_idx, line) in self.lines.iter().enumerate().skip(y + 1) {
            if let Some(idx) = line.find(ch) {
                return Some((idx, line_idx));
            }
        }

        None
    }

    pub fn get_previous_occurrence_of_char(
        &self,
        (x, y): (usize, usize),
        ch: char,
    ) -> Option<(usize, usize)> {
        // Search current line backwards from position
        if let Some(line) = self.lines.get(y) {
            if let Some(idx) = line[..x].rfind(ch) {
                return Some((idx, y));
            }
        }

        // Search previous lines backwards
        for line_idx in (0..y).rev() {
            if let Some(line) = self.lines.get(line_idx) {
                if let Some(idx) = line.rfind(ch) {
                    return Some((idx, line_idx));
                }
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
            if let Some(line) = self.lines.get(idx) {
                if line.trim().is_empty() {
                    return idx;
                }
            }
        }
        0
    }

    pub fn get_matching_paren_pos(&self, (x, y): (usize, usize)) -> Option<(usize, usize)> {
        let line = self.lines.get(y)?;
        let chars: Vec<char> = line.chars().collect();

        if x >= chars.len() {
            return None;
        }

        let start_char = chars[x];
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
                let current_chars: Vec<char> = current_line.chars().collect();

                for (char_idx, &ch) in current_chars.iter().enumerate().skip(start_pos) {
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
                let current_chars: Vec<char> = current_line.chars().collect();
                let end_pos = if line_idx == y {
                    x
                } else {
                    current_chars.len()
                };

                for char_idx in (0..end_pos).rev() {
                    let ch = current_chars[char_idx];
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
