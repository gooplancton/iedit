use iedit_document::{EditOperation, Text};
use termion::event::Key;

use crate::{
    Editor,
    editor::{
        commands::{CommandExecutionResult, EditorCommand, MoveCursor},
        modes::EditorMode,
    },
    input::Input,
};

impl Editor {
    pub fn insert_mode_execute_command(
        &mut self,
        command: EditorCommand,
    ) -> CommandExecutionResult {
        use CommandExecutionResult as R;
        use iedit_document::InverseStack as S;

        match command {
            EditorCommand::SwitchMode(mode) => {
                self.mode = mode;
            }
            EditorCommand::MoveCursor {
                movement,
                with_selection,
            } => {
                if !with_selection {
                    self.cursor.selection_anchor = None;
                } else if self.cursor.selection_anchor.is_none() {
                    self.cursor.selection_anchor = Some(self.cursor.pos())
                }

                match movement {
                    MoveCursor::AbsolutePos(pos) => self.cursor.update_pos(pos),
                    MoveCursor::Up(lines) => self.cursor.move_up(lines),
                    MoveCursor::Down(lines) => self.cursor.move_down(lines),
                    MoveCursor::Left(cols) => self.cursor.move_left(cols),
                    MoveCursor::Right(cols) => self.cursor.move_right(cols),
                    MoveCursor::NextWord => {
                        let next_word_pos = self.document.get_next_word_pos(self.cursor.pos());
                        self.cursor.update_pos(next_word_pos);
                    }
                    MoveCursor::PreviousWord => {
                        let previous_word_pos =
                            self.document.get_previous_word_pos(self.cursor.pos());
                        self.cursor.update_pos(previous_word_pos);
                    }
                    MoveCursor::NextParagraph => {
                        let next_paragraph_row =
                            self.document.get_next_blank_line_idx(self.cursor.cur_y);
                        self.cursor.update_pos((0, next_paragraph_row));
                    }
                    MoveCursor::PreviousParagraph => {
                        let previous_paragraph_row =
                            self.document.get_previous_blank_line_idx(self.cursor.cur_y);
                        self.cursor.update_pos((0, previous_paragraph_row));
                    }
                    MoveCursor::MatchingParenthesis => {
                        if let Some(pos) = self.document.get_matching_paren_pos(self.cursor.pos()) {
                            self.cursor.update_pos(pos);
                        }
                    }
                    MoveCursor::NextOccurrenceOf(ch) => {
                        if let Some(pos) = self
                            .document
                            .get_next_occurrence_of_char(self.cursor.pos(), ch)
                        {
                            self.cursor.update_pos(pos);
                        }
                    }
                    MoveCursor::PreviousOccurrenceOf(ch) => {
                        if let Some(pos) = self
                            .document
                            .get_previous_occurrence_of_char(self.cursor.pos(), ch)
                        {
                            self.cursor.update_pos(pos);
                        }
                    }
                    MoveCursor::StartOfLine => {
                        self.cursor.update_pos((0, self.cursor.cur_y));
                    }
                    MoveCursor::EndOfLine => {
                        self.cursor.update_pos((usize::MAX, 0));
                    }
                    MoveCursor::StartOfFile => {
                        self.cursor.update_pos((0, 0));
                    }
                    MoveCursor::EndOfFile => {
                        self.cursor.update_pos((0, self.document.n_lines()));
                    }
                }
            }
            EditorCommand::ClearSelection => {
                self.is_selection_locked = false;
                self.cursor.selection_anchor = None;
            }
            EditorCommand::Edit(op) => {
                if let Some(new_pos) = self.document.apply_edit(op, S::Undo) {
                    self.cursor.update_pos(new_pos);
                }

                self.first_quit_sent = false;
                self.cursor.selection_anchor = None;
            }
            EditorCommand::UndoLastEdit => {
                if let Some(new_pos) = self.document.undo_last_edit() {
                    self.cursor.update_pos(new_pos);
                }

                self.first_quit_sent = false;
                self.cursor.selection_anchor = None;
            }
            EditorCommand::RedoLastEdit => {
                if let Some(new_pos) = self.document.redo_last_edit() {
                    self.cursor.update_pos(new_pos);
                }

                self.first_quit_sent = false;
                self.cursor.selection_anchor = None;
            }
            EditorCommand::FindMatchForward => todo!(),
            EditorCommand::FindMatchBackward => todo!(),
            EditorCommand::ExecuteFile(executor_key) => {
                self.execute_file(executor_key);
            }
            _ => {}
        }

        R::Continue
    }

    pub fn insert_mode_parse_command(&self, input: Input) -> Option<EditorCommand> {
        use EditOperation as Op;
        use EditorCommand as C;
        use EditorMode as M;
        use Text as T;

        match input {
            Input::Keypress(Key::Esc) => Some(C::ClearSelection),
            Input::Keypress(Key::Ctrl('z')) => Some(C::UndoLastEdit),
            Input::Keypress(Key::Ctrl('r')) => Some(C::RedoLastEdit),
            Input::Keypress(Key::Ctrl('f')) => Some(C::SwitchMode(M::Search)),
            Input::Keypress(Key::Ctrl('g')) => Some(C::SwitchMode(M::Goto(self.cursor.pos()))),
            Input::Keypress(Key::Ctrl('l')) => Some(C::ToggleLockSelection),
            Input::Keypress(Key::CtrlDown) => Some(C::ScrollViewportDown),
            Input::Keypress(Key::CtrlUp) => Some(C::ScrollViewportUp),
            Input::Keypress(Key::Ctrl('d')) => Some(C::MoveCursor {
                movement: MoveCursor::Down(self.config.page_size),
                with_selection: self.is_selection_locked,
            }),
            Input::Keypress(Key::Ctrl('u')) => Some(C::MoveCursor {
                movement: MoveCursor::Up(self.config.page_size),
                with_selection: self.is_selection_locked,
            }),
            Input::Keypress(Key::Ctrl('}')) => Some(C::MoveCursor {
                movement: MoveCursor::NextParagraph,
                with_selection: self.is_selection_locked,
            }),
            Input::Keypress(Key::Ctrl('p')) => Some(C::MoveCursor {
                movement: MoveCursor::MatchingParenthesis,
                with_selection: self.is_selection_locked,
            }),
            Input::Keypress(Key::Char(ch)) => match self.cursor.get_highlighted_range() {
                None => Some(C::Edit(Op::Insertion {
                    pos: self.cursor.pos(),
                    text: T::Char(ch),
                })),
                Some((pos_from, pos_to)) => Some(C::Edit(Op::Replacement {
                    pos_from,
                    pos_to,
                    text: Text::Char(ch),
                })),
            },
            Input::Keypress(Key::Backspace) | Input::Keypress(Key::Delete) => {
                match self.cursor.get_highlighted_range() {
                    None => Some(C::Edit(Op::Deletion {
                        pos: self.cursor.pos(),
                    })),
                    Some((pos_from, pos_to)) => Some(C::Edit(Op::Replacement {
                        pos_from,
                        pos_to,
                        text: Text::Empty,
                    })),
                }
            }
            Input::Keypress(Key::Ctrl('h')) | Input::Keypress(Key::Ctrl('\x7F')) => {
                match self.cursor.get_highlighted_range() {
                    None => {
                        let word_start_pos = self.document.get_previous_word_pos(self.cursor.pos());
                        Some(C::Edit(Op::Replacement {
                            pos_from: word_start_pos,
                            pos_to: self.cursor.pos(),
                            text: Text::Empty,
                        }))
                    }
                    Some((pos_from, pos_to)) => Some(C::Edit(Op::Replacement {
                        pos_from,
                        pos_to,
                        text: Text::Empty,
                    })),
                }
            }
            Input::Keypress(Key::Left) => Some(C::MoveCursor {
                with_selection: self.is_selection_locked,
                movement: MoveCursor::Left(1),
            }),
            Input::Keypress(Key::Right) => Some(C::MoveCursor {
                with_selection: self.is_selection_locked,
                movement: MoveCursor::Right(1),
            }),
            Input::Keypress(Key::Up) => Some(C::MoveCursor {
                with_selection: self.is_selection_locked,
                movement: MoveCursor::Up(1),
            }),
            Input::Keypress(Key::Down) => Some(C::MoveCursor {
                with_selection: self.is_selection_locked,
                movement: MoveCursor::Down(1),
            }),
            Input::Keypress(Key::ShiftLeft) => Some(C::MoveCursor {
                with_selection: true,
                movement: MoveCursor::Left(1),
            }),
            Input::Keypress(Key::ShiftRight) => Some(C::MoveCursor {
                with_selection: true,
                movement: MoveCursor::Right(1),
            }),
            Input::Keypress(Key::ShiftUp) => Some(C::MoveCursor {
                with_selection: true,
                movement: MoveCursor::Up(1),
            }),
            Input::Keypress(Key::ShiftDown) => Some(C::MoveCursor {
                with_selection: true,
                movement: MoveCursor::Down(1),
            }),
            Input::Keypress(Key::Alt('f')) | Input::Keypress(Key::CtrlRight) => {
                Some(C::MoveCursor {
                    with_selection: self.is_selection_locked,
                    movement: MoveCursor::NextWord,
                })
            }
            Input::Keypress(Key::Alt('b')) | Input::Keypress(Key::CtrlLeft) => {
                Some(C::MoveCursor {
                    with_selection: self.is_selection_locked,
                    movement: MoveCursor::PreviousWord,
                })
            }
            Input::KeyChord([Key::Ctrl('k'), Key::Char('l'), Key::Char('n')]) => {
                Some(C::ToggleLineNumbers)
            }
            Input::KeyChord([Key::Ctrl('k'), Key::Char('t'), Key::Char(ch)]) => {
                Some(C::MoveCursor {
                    movement: MoveCursor::NextOccurrenceOf(ch),
                    with_selection: self.is_selection_locked,
                })
            }
            Input::KeyChord([Key::Ctrl('k'), Key::Char('T'), Key::Char(ch)]) => {
                Some(C::MoveCursor {
                    movement: MoveCursor::PreviousOccurrenceOf(ch),
                    with_selection: self.is_selection_locked,
                })
            }
            Input::KeyChord([Key::Ctrl('k'), Key::Char('x'), executor_key]) => {
                Some(C::ExecuteFile(executor_key))
            }
            _ => None,
        }
    }
}

