use iedit_document::{EditOperation, Text};
use termion::event::Key;

use crate::{
    Editor,
    editor::{
        commands::{
            CommandExecutionResult, CursorMovement, EditorCommand, send_simple_notification,
        },
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
                        self.cursor.update_pos((0, next_paragraph_row));
                    }
                    CursorMovement::PreviousParagraph => {
                        let previous_paragraph_row =
                            self.document.get_previous_blank_line_idx(self.cursor.cur_y);
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
            EditorCommand::ClearSelection => {
                self.is_selection_locked = false;
                self.needs_full_rerender = true;
                self.cursor.selection_anchor = None;
                self.search_item = None;
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
            EditorCommand::FindMatchForward | EditorCommand::FindMatchBackward => {
                let (x_from, x_to) = if self.cursor.selection_anchor.is_some() {
                    let (pos_from, pos_to) = self.cursor.get_highlighted_range().unwrap();
                    self.cursor.selection_anchor = None;
                    if pos_from.1 != pos_to.1 {
                        // TODO: implement this
                        send_simple_notification("Can't yet match across lines");
                        return R::Continue;
                    }
                    (pos_from.0, pos_to.0)
                } else if let Some(word_range) =
                    self.document.get_word_boundaries(self.cursor.pos())
                {
                    word_range
                } else {
                    return R::Continue;
                };

                let next_cursor_pos = self
                    .document
                    .lines
                    .get(self.cursor.cur_y)
                    .map(|line| line.get_range(x_from..=x_to))
                    .and_then(|lit| {
                        if matches!(command, EditorCommand::FindMatchForward) {
                            self.document
                                .get_next_literal_match_pos(self.cursor.pos(), lit)
                        } else {
                            self.document
                                .get_previous_literal_match_pos(self.cursor.pos(), lit)
                        }
                    });

                if let Some((next_x, next_y)) = next_cursor_pos {
                    self.cursor.update_pos((next_x, next_y));
                } else if matches!(command, EditorCommand::FindMatchForward) {
                    send_simple_notification("Already at last match");
                } else if matches!(command, EditorCommand::FindMatchBackward) {
                    send_simple_notification("Already at first match");
                }
            }
            EditorCommand::ExecuteFile(executor_key) => {
                self.execute_file(executor_key);
            }
            EditorCommand::ViewExecutionOutput => {
                self.toggle_execution_output();
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
            Input::Keypress(Key::Ctrl('f')) => Some(C::SwitchMode(M::Search {
                original_cursor_pos: self.cursor.pos(),
                is_backwards: false,
            })),
            Input::Keypress(Key::Ctrl('b')) => Some(C::SwitchMode(M::Search {
                original_cursor_pos: self.cursor.pos(),
                is_backwards: true,
            })),
            Input::Keypress(Key::Ctrl('g')) => Some(C::SwitchMode(M::Goto {
                original_cursor_pos: self.cursor.pos(),
            })),
            Input::Keypress(Key::Ctrl('l')) => Some(C::ToggleLockSelection),
            Input::Keypress(Key::CtrlDown) => Some(C::ScrollViewportDown),
            Input::Keypress(Key::CtrlUp) => Some(C::ScrollViewportUp),
            Input::Keypress(Key::Ctrl('d')) => Some(C::MoveCursor {
                movement: CursorMovement::Down(self.ui.editor_lines as usize),
                with_selection: self.is_selection_locked,
            }),
            Input::Keypress(Key::Ctrl('u')) => Some(C::MoveCursor {
                movement: CursorMovement::Up(self.ui.editor_lines as usize),
                with_selection: self.is_selection_locked,
            }),
            Input::Keypress(Key::Ctrl('e')) => Some(C::MoveCursor {
                movement: CursorMovement::NextParagraph,
                with_selection: self.is_selection_locked,
            }),
            Input::Keypress(Key::Ctrl('p')) => Some(C::MoveCursor {
                movement: CursorMovement::MatchingParenthesis,
                with_selection: self.is_selection_locked,
            }),
            Input::Keypress(Key::Ctrl('n')) => Some(C::FindMatchForward),
            Input::Keypress(Key::Char(ch)) => {
                let text = if ch == '\t' && self.config.tab_emit_spaces {
                    let n_spaces = self.config.tab_size as usize
                        - (self.cursor.cur_x % self.config.tab_size as usize);
                    T::String(" ".repeat(n_spaces))
                } else {
                    T::Char(ch)
                };
                match self.cursor.get_highlighted_range() {
                    None => Some(C::Edit(Op::Insertion {
                        pos: self.cursor.pos(),
                        text,
                    })),
                    Some((pos_from, pos_to)) => Some(C::Edit(Op::Replacement {
                        pos_from,
                        pos_to,
                        text,
                    })),
                }
            }
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
                movement: CursorMovement::Left(1),
            }),
            Input::Keypress(Key::Right) => Some(C::MoveCursor {
                with_selection: self.is_selection_locked,
                movement: CursorMovement::Right(1),
            }),
            Input::Keypress(Key::Up) => Some(C::MoveCursor {
                with_selection: self.is_selection_locked,
                movement: CursorMovement::Up(1),
            }),
            Input::Keypress(Key::Down) => Some(C::MoveCursor {
                with_selection: self.is_selection_locked,
                movement: CursorMovement::Down(1),
            }),
            Input::Keypress(Key::ShiftLeft) => Some(C::MoveCursor {
                with_selection: true,
                movement: CursorMovement::Left(1),
            }),
            Input::Keypress(Key::ShiftRight) => Some(C::MoveCursor {
                with_selection: true,
                movement: CursorMovement::Right(1),
            }),
            Input::Keypress(Key::ShiftUp) => Some(C::MoveCursor {
                with_selection: true,
                movement: CursorMovement::Up(1),
            }),
            Input::Keypress(Key::ShiftDown) => Some(C::MoveCursor {
                with_selection: true,
                movement: CursorMovement::Down(1),
            }),
            Input::Keypress(Key::Alt('f')) | Input::Keypress(Key::CtrlRight) => {
                Some(C::MoveCursor {
                    with_selection: self.is_selection_locked,
                    movement: CursorMovement::NextWord,
                })
            }
            Input::Keypress(Key::Alt('b')) | Input::Keypress(Key::CtrlLeft) => {
                Some(C::MoveCursor {
                    with_selection: self.is_selection_locked,
                    movement: CursorMovement::PreviousWord,
                })
            }
            Input::KeyChord([Key::Ctrl('k'), Key::Char('l'), Key::Char('n')]) => {
                Some(C::ToggleLineNumbers)
            }
            Input::KeyChord([Key::Ctrl('k'), Key::Char('l'), Key::Char('d')]) => {
                Some(C::Edit(Op::Replacement {
                    pos_from: (0, self.cursor.cur_y),
                    pos_to: (0, self.cursor.cur_y + 1),
                    text: Text::Empty,
                }))
            }
            Input::KeyChord([Key::Ctrl('k'), Key::Char('t'), Key::Char(ch)]) => {
                Some(C::MoveCursor {
                    movement: CursorMovement::NextOccurrenceOf(ch),
                    with_selection: self.is_selection_locked,
                })
            }
            Input::KeyChord([Key::Ctrl('k'), Key::Char('T'), Key::Char(ch)]) => {
                Some(C::MoveCursor {
                    movement: CursorMovement::PreviousOccurrenceOf(ch),
                    with_selection: self.is_selection_locked,
                })
            }
            Input::KeyChord([Key::Ctrl('k'), Key::Char('x'), executor_key]) => {
                Some(C::ExecuteFile(executor_key))
            }
            Input::KeyChord([Key::Ctrl('k'), Key::Char('v'), Key::Char('o')]) => {
                Some(C::ViewExecutionOutput)
            }
            _ => None,
        }
    }
}
