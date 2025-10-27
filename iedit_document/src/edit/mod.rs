use crate::Document;

mod delete;
mod insert;

pub enum EditOperation {
    InsertCharAt {
        pos: (usize, usize),
        ch: char,
    },
    DeleteCharAt {
        pos: (usize, usize),
    },
    InsertStringAt {
        pos: (usize, usize),
        string: String,
    },
    DeleteRange {
        x_from: usize,
        x_to: usize,
        y: usize,
    },
    ReplaceRange {
        x_from: usize,
        x_to: usize,
        y: usize,
        string: String,
    },
    DeleteRangeMultiline {
        pos_from: (usize, usize),
        pos_to: (usize, usize),
    },
    InsertStringsAtMultiline {
        pos: (usize, usize),
        strings: Vec<String>,
    },
    ReplaceRangeMultiline {
        pos_from: (usize, usize),
        pos_to: (usize, usize),
        strings: Vec<String>,
    },
}

impl Document {
    pub fn apply_edit(&mut self, op: EditOperation) {
        use EditOperation as Op;

        match op {
            Op::InsertCharAt { pos, ch } => self.insert_char_at(pos, ch),
            Op::DeleteCharAt { pos } => self.delete_char_at(pos),
            Op::InsertStringAt { pos, string } => self.insert_string_at(pos, string),
            Op::DeleteRange { x_from, x_to, y } => todo!(),
            Op::ReplaceRange {
                x_from,
                x_to,
                y,
                string,
            } => todo!(),
            Op::DeleteRangeMultiline { pos_from, pos_to } => todo!(),
            Op::InsertStringsAtMultiline { pos, strings } => todo!(),
            Op::ReplaceRangeMultiline {
                pos_from,
                pos_to,
                strings,
            } => todo!(),
        }
    }
}
