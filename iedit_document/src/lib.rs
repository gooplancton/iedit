mod edit;
mod line;

pub use edit::EditOperation;
pub use line::{CharacterEditable, DocumentLine};

pub struct Document {
    pub lines: Vec<String>,
    undo_stack: Vec<EditOperation>,
    redo_stack: Vec<EditOperation>,
    pub has_been_edited: bool,
}

impl Document {
    pub fn new(lines: Vec<String>) -> Self {
        Self {
            lines,
            undo_stack: vec![],
            redo_stack: vec![],
            has_been_edited: false,
        }
    }

    #[inline]
    pub fn n_lines(&self) -> usize {
        self.lines.len()
    }

    #[inline]
    pub fn get_or_add_line(&mut self, y: usize) -> &mut String {
        if y < self.n_lines() {
            self.lines.get_mut(y).unwrap()
        } else {
            self.lines.push(String::new());
            self.lines.last_mut().unwrap()
        }
    }

    pub fn get_char_at_pos(&self, pos: (usize, usize)) -> Option<char> {
        todo!()
    }

    pub fn get_next_word_x(&self, pos: (usize, usize)) ->  usize {
        todo!()
    }

    pub fn get_previous_word_x(&self, pos: (usize, usize)) -> usize {
        todo!()
    }

    pub fn get_word_start_x(&self, pos: (usize, usize)) -> usize {
        todo!()
    }

    pub fn get_next_occurrence_of_char(
        &self,
        pos: (usize, usize),
        ch: char,
    ) -> Option<(usize, usize)> {
        todo!()
    }

    pub fn get_previous_occurrence_of_char(
        &self,
        pos: (usize, usize),
        ch: char,
    ) -> Option<(usize, usize)> {
        todo!()
    }

    pub fn get_next_blank_line_idx(&self, pos_y: usize) -> usize {
        todo!()
    }

    pub fn get_previous_blank_line_idx(&self, pos_y: usize) -> usize {
        todo!()
    }

    pub fn get_matching_paren_pos(&self, pos: (usize, usize)) -> Option<(usize, usize)> {
        todo!()
    }
}
