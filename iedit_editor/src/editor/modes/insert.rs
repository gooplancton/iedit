use iedit_document::{CharacterIndexable, DocumentLine, EditOperation, Text};
use termion::event::Key;

use crate::{
    Editor,
    editor::{
        commands::{
            CommandExecutionResult, CursorMovement, EditorCommand, Executor,
            send_simple_notification,
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
            EditorCommand::AutocompleteDisplay => {
                self.is_autocomplete_open = true;
            }
            EditorCommand::AutocompleteNext => {
                self.autocomplete.selected_idx += 1;
            }
            EditorCommand::AutocompletePrevious => {
                self.autocomplete.selected_idx = self.autocomplete.selected_idx.saturating_sub(1);
            }
            EditorCommand::AutocompleteInsert => {
                let text = Text::String(self.autocomplete.get_selected_choice());
                let op = EditOperation::Insertion {
                    pos: self.cursor.pos(),
                    text,
                };

                if let Some(new_pos) = self.document.apply_edit(op, S::Undo) {
                    self.cursor.update_pos(new_pos, false);
                }

                self.first_quit_sent = false;
                self.cursor.selection_anchor = None;
                self.is_autocomplete_open = false;
            }
            EditorCommand::SwitchMode(mode) => {
                self.mode = mode;
            }
            EditorCommand::OpenCommandLine => {
                self.prompt_user("> ", move |editor, cmd| {
                    editor.execute_from_cmd_prompt(cmd.as_ref())
                });
            }
            EditorCommand::PromptExecutor => {
                self.prompt_user("Run with: ", move |editor, executor| {
                    editor.execute_file(Executor::Literal(executor.into()));
                    CommandExecutionResult::Continue
                });
            }
            EditorCommand::Paste => {
                if let Some(yanked_text) = self.clipboard.get_text() {
                    match &yanked_text {
                        Text::String(string) => send_simple_notification(format!(
                            "Pasted {} characters",
                            string.n_chars()
                        )),
                        Text::Lines(lines) => {
                            send_simple_notification(format!("Pasted {} lines", lines.len()))
                        }
                        _ => {}
                    };

                    let edit = match self.cursor.get_selected_range() {
                        Some((pos_from, pos_to)) => EditOperation::Replacement {
                            pos_from,
                            pos_to,
                            text: yanked_text,
                        },
                        None => EditOperation::Insertion {
                            pos: self.cursor.pos(),
                            text: yanked_text,
                        },
                    };

                    self.cursor.selection_anchor = None;
                    if let Some(cursor_pos) = self.document.apply_edit(edit, S::Undo) {
                        self.cursor.update_pos(cursor_pos, false);
                    }
                }
            }
            EditorCommand::YankSelection | EditorCommand::CutSelection => {
                if let Some((pos_from, pos_to)) = self.cursor.get_selected_range() {
                    let text = if pos_from.1 == pos_to.1 {
                        let string = self.document.lines[pos_from.1]
                            .get_range(pos_from.0..pos_to.0)
                            .to_string();

                        send_simple_notification(format!("Yanked {} characters", string.n_chars()));
                        Text::String(string)
                    } else {
                        let mut lines = vec![];
                        for line_idx in pos_from.1..=pos_to.1 {
                            let line = if line_idx == pos_from.1 {
                                self.document
                                    .lines
                                    .get(line_idx)
                                    .map(|line| line.get_range(pos_from.0..))
                            } else if line_idx == pos_to.1 {
                                self.document
                                    .lines
                                    .get(line_idx)
                                    .map(|line| line.get_range(..pos_to.0))
                            } else {
                                self.document.lines.get(line_idx).map(DocumentLine::as_ref)
                            };

                            if let Some(line) = line {
                                lines.push(line.to_owned());
                            }
                        }

                        send_simple_notification(format!("Yanked {} lines", lines.len()));
                        Text::Lines(lines)
                    };

                    self.clipboard.set_text(text);
                    if matches!(command, EditorCommand::CutSelection) {
                        self.cursor.selection_anchor = None;
                        self.cursor.update_pos(pos_from, false);
                        self.document.apply_edit(
                            EditOperation::Replacement {
                                pos_from,
                                pos_to,
                                text: Text::Empty,
                            },
                            S::Undo,
                        );
                    }
                }
            }
            EditorCommand::ClearSelection => {
                self.is_autocomplete_open = false;
                self.is_selection_locked = false;
                self.needs_full_rerender = true;
                self.cursor.selection_anchor = None;
                self.matched_range = None;
                self.search_item = None;
            }
            EditorCommand::Edit(op) => {
                let mut is_typing = matches!(
                    op,
                    EditOperation::Insertion {
                        text: Text::Char(_),
                        ..
                    }
                );

                if let Some(new_pos) = self.document.apply_edit(op, S::Undo) {
                    if new_pos.1 != self.cursor.cur_y {
                        is_typing = false;
                    }

                    self.cursor.update_pos(new_pos, false);
                }

                self.first_quit_sent = false;
                self.cursor.selection_anchor = None;
                if is_typing {
                    let word_boundaries = self.document.get_word_boundaries((
                        self.cursor.cur_x.saturating_sub(1),
                        self.cursor.cur_y,
                    ));
                    let left_boundary = word_boundaries.map(|(x, _)| x).unwrap_or_default();
                    self.is_autocomplete_open = true;
                    self.update_autocomplete_choices(left_boundary);
                }
            }
            EditorCommand::UndoLastEdit => {
                if let Some(new_pos) = self.document.undo_last_edit() {
                    self.cursor.update_pos(new_pos, false);
                }

                self.first_quit_sent = false;
                self.cursor.selection_anchor = None;
            }
            EditorCommand::RedoLastEdit => {
                if let Some(new_pos) = self.document.redo_last_edit() {
                    self.cursor.update_pos(new_pos, false);
                }

                self.first_quit_sent = false;
                self.cursor.selection_anchor = None;
            }
            EditorCommand::FindMatchForward | EditorCommand::FindMatchBackward => {
                let (x_from, x_to) = if self.cursor.selection_anchor.is_some() {
                    let (pos_from, pos_to) = self.cursor.get_selected_range().unwrap();
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

                if let Some((start, end)) = next_cursor_pos {
                    self.matched_range = Some((start, end));
                    self.cursor.update_pos(start, false);
                } else if matches!(command, EditorCommand::FindMatchForward) {
                    send_simple_notification("Already at last match");
                } else if matches!(command, EditorCommand::FindMatchBackward) {
                    send_simple_notification("Already at first match");
                }
            }
            EditorCommand::ExecuteFile(executor_key) => {
                self.execute_file(Executor::Key(executor_key));
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
            Input::Keypress(Key::AltDown) => Some(C::ScrollViewportDown),
            Input::Keypress(Key::AltUp) => Some(C::ScrollViewportUp),
            Input::Keypress(Key::Alt('i')) => Some(C::MoveCursor {
                movement: CursorMovement::NextJump,
                with_selection: self.is_selection_locked,
            }),
            Input::Keypress(Key::Alt('o')) => Some(C::MoveCursor {
                movement: CursorMovement::PreviousJump,
                with_selection: self.is_selection_locked,
            }),
            Input::Keypress(Key::Ctrl('d')) | Input::Keypress(Key::PageDown) => {
                Some(C::MoveCursor {
                    movement: CursorMovement::Down(self.ui.editor_lines as usize),
                    with_selection: self.is_selection_locked,
                })
            }
            Input::Keypress(Key::Ctrl('u')) => Some(C::MoveCursor {
                movement: CursorMovement::Up(self.ui.editor_lines as usize),
                with_selection: self.is_selection_locked,
            }),
            Input::Keypress(Key::Alt('a')) => Some(C::MoveCursor {
                movement: CursorMovement::StartOfLine,
                with_selection: self.is_selection_locked,
            }),
            Input::Keypress(Key::Alt('s')) => Some(C::MoveCursor {
                movement: CursorMovement::EndOfLine,
                with_selection: self.is_selection_locked,
            }),
            Input::Keypress(Key::CtrlDown) => Some(C::MoveCursor {
                movement: CursorMovement::NextParagraph,
                with_selection: self.is_selection_locked,
            }),
            Input::Keypress(Key::CtrlShiftDown) => Some(C::MoveCursor {
                movement: CursorMovement::NextParagraph,
                with_selection: true,
            }),
            Input::Keypress(Key::CtrlUp) => Some(C::MoveCursor {
                movement: CursorMovement::PreviousParagraph,
                with_selection: self.is_selection_locked,
            }),
            Input::Keypress(Key::CtrlShiftUp) => Some(C::MoveCursor {
                movement: CursorMovement::PreviousParagraph,
                with_selection: true,
            }),
            Input::Keypress(Key::Alt('p')) => Some(C::MoveCursor {
                movement: CursorMovement::MatchingParenthesis,
                with_selection: self.is_selection_locked,
            }),
            Input::Keypress(Key::Alt('n')) => Some(C::FindMatchForward),
            Input::Keypress(Key::Alt('m')) => Some(C::FindMatchBackward),
            Input::Keypress(Key::Ctrl('e')) | Input::Keypress(Key::Alt('e')) => {
                Some(C::OpenCommandLine)
            }
            Input::Keypress(Key::Ctrl('y')) => Some(C::YankSelection),
            Input::Keypress(Key::Ctrl('x')) => Some(C::CutSelection),
            Input::Keypress(Key::Ctrl('p')) => Some(C::Paste),
            Input::Keypress(Key::Ctrl('m')) => Some(C::AutocompletePrevious),
            Input::Keypress(Key::Ctrl('n')) => {
                if self.is_autocomplete_open {
                    Some(C::AutocompleteNext)
                } else {
                    Some(C::AutocompleteDisplay)
                }
            }
            Input::Keypress(Key::Char(ch)) => {
                if ch == '\n' && self.is_autocomplete_open && !self.autocomplete.choices.is_empty()
                {
                    return Some(C::AutocompleteInsert);
                }

                let text = if ch == '\t' && self.config.tab_emit_spaces {
                    let n_spaces = self.config.tab_size as usize
                        - (self.cursor.cur_x % self.config.tab_size as usize);
                    T::String(" ".repeat(n_spaces))
                } else {
                    T::Char(ch)
                };
                match self.cursor.get_selected_range() {
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
                match self.cursor.get_selected_range() {
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
            Input::Keypress(Key::Ctrl('h'))
            | Input::Keypress(Key::Ctrl('\x7F'))
            | Input::Keypress(Key::Alt('\x7F')) => {
                if self.cursor.cur_x == 0 {
                    Some(C::Edit(Op::Deletion {
                        pos: self.cursor.pos(),
                    }))
                } else {
                    match self.cursor.get_selected_range() {
                        None => {
                            let word_start_pos =
                                self.document.get_previous_word_start_pos(self.cursor.pos());
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
            }
            Input::Keypress(Key::Left) | Input::Keypress(Key::Alt('h')) => Some(C::MoveCursor {
                with_selection: self.is_selection_locked,
                movement: CursorMovement::Left(1),
            }),
            Input::Keypress(Key::Right) | Input::Keypress(Key::Alt('l')) => Some(C::MoveCursor {
                with_selection: self.is_selection_locked,
                movement: CursorMovement::Right(1),
            }),
            Input::Keypress(Key::Up) | Input::Keypress(Key::Alt('k')) => Some(C::MoveCursor {
                with_selection: self.is_selection_locked,
                movement: CursorMovement::Up(1),
            }),
            Input::Keypress(Key::Down) | Input::Keypress(Key::Alt('j')) => Some(C::MoveCursor {
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
            Input::Keypress(Key::CtrlRight) | Input::Keypress(Key::Alt('w')) => {
                Some(C::MoveCursor {
                    with_selection: self.is_selection_locked,
                    movement: CursorMovement::NextWordEnd,
                })
            }
            Input::Keypress(Key::CtrlShiftRight) => Some(C::MoveCursor {
                with_selection: true,
                movement: CursorMovement::NextWordEnd,
            }),
            Input::Keypress(Key::CtrlLeft) | Input::Keypress(Key::Alt('b')) => {
                Some(C::MoveCursor {
                    with_selection: self.is_selection_locked,
                    movement: CursorMovement::PreviousWordStart,
                })
            }
            Input::Keypress(Key::CtrlShiftLeft) => Some(C::MoveCursor {
                with_selection: true,
                movement: CursorMovement::PreviousWordStart,
            }),
            Input::Keypress(Key::Ctrl('t')) => Some(EditorCommand::DisplayHelp),
            Input::KeyChord([Key::Ctrl('k'), Key::Null, Key::Null]) => {
                Some(EditorCommand::DisplayChordsHelp)
            }
            Input::KeyChord([Key::Ctrl('k'), Key::Char('x'), Key::Null]) => {
                Some(EditorCommand::DisplayExecuteChordHelp)
            }
            Input::KeyChord([Key::Ctrl('k'), Key::Char('l'), Key::Null]) => {
                Some(EditorCommand::DisplayLineChordHelp)
            }
            Input::KeyChord([Key::Ctrl('k'), Key::Char('v'), Key::Null]) => {
                Some(EditorCommand::DisplayViewChordHelp)
            }
            Input::KeyChord([Key::Ctrl('k'), Key::Char('t') | Key::Char('T'), Key::Null]) => {
                Some(EditorCommand::DisplayPressCharacterPopup)
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
            Input::KeyChord([Key::Ctrl('k'), Key::Char('l'), Key::Char('w')]) => {
                Some(C::MoveCursor {
                    movement: CursorMovement::StartOfLine,
                    with_selection: self.is_selection_locked,
                })
            }
            Input::KeyChord([Key::Ctrl('k'), Key::Char('l'), Key::Char('e')]) => {
                Some(C::MoveCursor {
                    movement: CursorMovement::EndOfLine,
                    with_selection: self.is_selection_locked,
                })
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
            Input::KeyChord([Key::Ctrl('k'), Key::Char('x'), Key::Char('?')]) => {
                Some(C::PromptExecutor)
            }
            Input::KeyChord([Key::Ctrl('k'), Key::Char('x'), executor_key]) => {
                Some(C::ExecuteFile(executor_key))
            }
            Input::KeyChord([Key::Ctrl('k'), Key::Char('v'), Key::Char('o')]) => {
                Some(C::ViewExecutionOutput)
            }
            Input::KeyChord([Key::Ctrl('k'), Key::Char('s'), Key::Null]) => {
                Some(C::DisplaySelectionChordHelp)
            }
            Input::KeyChord([Key::Ctrl('k'), Key::Char('s'), Key::Char('l')]) => {
                Some(C::ToggleLockSelection)
            }
            _ => None,
        }
    }
}
