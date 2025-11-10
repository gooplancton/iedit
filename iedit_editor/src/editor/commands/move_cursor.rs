use crate::{Editor, editor::commands::EditorCommand};

pub enum CursorMovement {
    AbsolutePos((usize, usize)),
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
    NextWord,
    PreviousWord,
    NextParagraph,
    StartOfLine,
    EndOfLine,
    StartOfFile,
    EndOfFile,
    PreviousParagraph,
    MatchingParenthesis,
    NextOccurrenceOf(char),
    PreviousOccurrenceOf(char),
}

impl Editor {
    pub fn execute_cursor_movement_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::MoveCursor {
                movement,
                with_selection,
            } => {
                if !with_selection && self.cursor.selection_anchor.is_some() {
                    self.needs_full_rerender = true;
                    self.cursor.selection_anchor = None;
                } else if with_selection && self.cursor.selection_anchor.is_none() {
                    self.cursor.selection_anchor = Some(self.cursor.pos())
                }

                match movement {
                    CursorMovement::AbsolutePos(pos) => self.cursor.update_pos(pos),
                    CursorMovement::Up(lines) => self.cursor.move_up(lines),
                    CursorMovement::Down(lines) => self.cursor.move_down(lines),
                    CursorMovement::Left(cols) => self.cursor.move_left(cols),
                    CursorMovement::Right(cols) => self.cursor.move_right(cols),
                    CursorMovement::NextWord => {
                        let next_word_pos = self.document.get_next_word_pos(self.cursor.pos());
                        self.cursor.update_pos(next_word_pos);
                    }
                    CursorMovement::PreviousWord => {
                        let previous_word_pos =
                            self.document.get_previous_word_pos(self.cursor.pos());
                        self.cursor.update_pos(previous_word_pos);
                    }
                    CursorMovement::NextParagraph => {
                        let next_paragraph_row =
                            self.document.get_next_blank_line_idx(self.cursor.cur_y);
                        self.needs_full_rerender = true;
                        self.cursor.update_pos((0, next_paragraph_row));
                    }
                    CursorMovement::PreviousParagraph => {
                        let previous_paragraph_row =
                            self.document.get_previous_blank_line_idx(self.cursor.cur_y);
                        self.needs_full_rerender = true;
                        self.cursor.update_pos((0, previous_paragraph_row));
                    }
                    CursorMovement::MatchingParenthesis => {
                        if let Some(pos) = self.document.get_matching_paren_pos(self.cursor.pos()) {
                            self.cursor.update_pos(pos);
                        }
                    }
                    CursorMovement::NextOccurrenceOf(ch) => {
                        if let Some(pos) = self
                            .document
                            .get_next_occurrence_of_char(self.cursor.pos(), ch)
                        {
                            self.cursor.update_pos(pos);
                        }
                    }
                    CursorMovement::PreviousOccurrenceOf(ch) => {
                        if let Some(pos) = self
                            .document
                            .get_previous_occurrence_of_char(self.cursor.pos(), ch)
                        {
                            self.cursor.update_pos(pos);
                        }
                    }
                    CursorMovement::StartOfLine => {
                        self.cursor.update_pos((0, self.cursor.cur_y));
                    }
                    CursorMovement::EndOfLine => {
                        self.cursor.update_pos((usize::MAX, 0));
                    }
                    CursorMovement::StartOfFile => {
                        self.cursor.update_pos((0, 0));
                    }
                    CursorMovement::EndOfFile => {
                        self.cursor.update_pos((0, self.document.n_lines()));
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}
