use crate::Document;

mod delete;
mod insert;

pub enum Text {
    Empty,
    Char(char),
    String(String),
    Lines(Vec<String>),
}

impl Default for Text {
    fn default() -> Self {
        Self::Empty
    }
}

pub enum EditOperation {
    Insertion {
        pos: (usize, usize),
        text: Text,
    },
    Deletion {
        pos: (usize, usize),
    },
    Replacement {
        pos_from: (usize, usize),
        pos_to: (usize, usize),
        text: Text,
    },
}

// TODO: possibly other side effects?
pub type EditResult = Option<(usize, usize)>;

impl Document {
    pub fn apply_edit(&mut self, op: EditOperation) -> EditResult {
        use EditOperation as Op;
        use Text as T;

        match op {
            Op::Insertion {
                pos,
                text: T::Char(newline),
            } if newline == '\n' || newline == '\r' => self.insert_newline_at(pos),
            Op::Insertion {
                pos,
                text: T::Char(ch),
            } => self.insert_char_at(pos, ch),
            Op::Insertion {
                pos,
                text: T::String(string),
            } => self.insert_string_at(pos, string),
            Op::Insertion {
                pos,
                text: T::Lines(lines),
            } => self.insert_lines_at(pos, lines),
            Op::Deletion { pos } => self.delete_char_at(pos),
            Op::Replacement {
                pos_from,
                pos_to,
                text,
            } => {
                // NOTE: placeholder implementation
                self.delete_range(pos_from, pos_to);
                self.apply_edit(EditOperation::Insertion {
                    pos: pos_from,
                    text,
                })
            }
            _ => None,
        }
    }
}
