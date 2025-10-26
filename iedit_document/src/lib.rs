mod edit;
mod line;

pub use edit::Edit;
pub use line::{CharacterEditable, DocumentLine};

pub struct Document {
    pub lines: Vec<String>,
    undo_stack: Vec<Edit>,
    redo_stack: Vec<Edit>,
}

impl Document {
    pub fn new(lines: Vec<String>) -> Self {
        Self {
            lines,
            undo_stack: vec![],
            redo_stack: vec![],
        }
    }

    #[inline]
    pub fn n_lines(&self) -> usize {
        self.lines.len()
    }

    pub fn get_char_at_pos(&self, pos: (usize, usize)) -> Option<char> {
        todo!()
    }
}
