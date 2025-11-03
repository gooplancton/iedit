use crate::Document;

mod delete;
mod insert;

#[derive(Debug)]
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

#[derive(Debug)]
pub enum EditOperation {
    LineRemoval {
        idx: usize,
    },
    Deletion {
        pos: (usize, usize),
    },
    Insertion {
        pos: (usize, usize),
        text: Text,
    },
    Replacement {
        pos_from: (usize, usize),
        pos_to: (usize, usize),
        text: Text,
    },
}

/// Cursor position following edit
pub type EditResult = Option<(usize, usize)>;

pub enum InverseStack {
    Undo,
    Redo,
}

impl Document {
    pub fn apply_edit(&mut self, op: EditOperation, inverse_stack: InverseStack) -> EditResult {
        if self.is_readonly {
            return None;
        }

        use EditOperation as Op;
        use Text as T;

        if matches!(inverse_stack, InverseStack::Undo) {
            self.redo_stack.clear();
        }

        self.has_been_edited = true;

        match op {
            Op::Insertion {
                pos,
                text: T::Char(newline),
            } if newline == '\n' || newline == '\r' => {
                let new_pos = self.insert_newline_at(pos)?;
                self.get_inverse_stack(inverse_stack)
                    .push(Op::Deletion { pos: new_pos });

                Some(new_pos)
            }
            Op::Insertion {
                pos,
                text: T::Char(ch),
            } => {
                let new_pos = self.insert_char_at(pos, ch)?;
                self.get_inverse_stack(inverse_stack)
                    .push(Op::Deletion { pos: new_pos });

                Some(new_pos)
            }
            Op::Insertion {
                pos,
                text: T::String(string),
            } => {
                let new_pos = self.insert_string_at(pos, string)?;
                self.get_inverse_stack(inverse_stack).push(Op::Replacement {
                    pos_from: pos,
                    pos_to: new_pos,
                    text: T::Empty,
                });

                Some(new_pos)
            }
            Op::Insertion {
                pos,
                text: T::Lines(lines),
            } => {
                let new_pos = self.insert_lines_at(pos, lines)?;
                self.get_inverse_stack(inverse_stack).push(Op::Replacement {
                    pos_from: pos,
                    pos_to: new_pos,
                    text: T::Empty,
                });

                Some(new_pos)
            }
            Op::Deletion { pos } => match self.delete_char_at(pos) {
                (newline, Some(new_pos)) if newline == '\n' || newline == '\r' => {
                    self.get_inverse_stack(inverse_stack).push(Op::Insertion {
                        pos: new_pos,
                        text: T::Char(newline),
                    });

                    Some(new_pos)
                }
                (ch, Some(new_pos)) => {
                    self.get_inverse_stack(inverse_stack).push(Op::Insertion {
                        pos,
                        text: T::Char(ch),
                    });

                    Some(new_pos)
                }
                (_, None) => None,
            },
            Op::Replacement {
                pos_from,
                pos_to,
                text,
            } => {
                let deleted_text = self.delete_range(pos_from, pos_to);
                let new_pos = match text {
                    T::Empty => Some(pos_from),
                    T::Char(ch) => self.insert_char_at(pos_from, ch),
                    T::String(string) => self.insert_string_at(pos_from, string),
                    T::Lines(lines) => self.insert_lines_at(pos_from, lines),
                }?;

                self.get_inverse_stack(inverse_stack).push(Op::Replacement {
                    pos_from,
                    pos_to: new_pos,
                    text: deleted_text,
                });

                Some(new_pos)
            }
            _ => None,
        }
    }

    pub fn undo_last_edit(&mut self) -> EditResult {
        let op = self.undo_stack.pop()?;

        self.apply_edit(op, InverseStack::Redo)
    }

    pub fn redo_last_edit(&mut self) -> EditResult {
        let op = self.redo_stack.pop()?;

        self.apply_edit(op, InverseStack::Undo)
    }

    pub fn get_inverse_stack(
        &mut self,
        inverse_stack: InverseStack,
    ) -> &mut std::vec::Vec<EditOperation> {
        let inverse_stack: &mut Vec<EditOperation> = match inverse_stack {
            InverseStack::Undo => self.undo_stack.as_mut(),
            InverseStack::Redo => self.redo_stack.as_mut(),
        };

        inverse_stack
    }
}
