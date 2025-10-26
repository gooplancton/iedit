use iedit_document::Edit;
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
        command: &EditorCommand,
    ) -> CommandExecutionResult {
        todo!()
    }

    pub fn insert_mode_parse_command(&self, input: Input) -> Option<EditorCommand> {
        use Edit as E;
        use EditorCommand as C;
        use EditorMode as M;

        match input {
            Input::Keypress(Key::Esc) => Some(C::ClearSelection),
            Input::Keypress(Key::Ctrl('q')) => Some(C::Quit),
            Input::Keypress(Key::Ctrl('s')) => Some(C::Save),
            Input::Keypress(Key::Ctrl('z')) => Some(C::UndoLastEdit),
            Input::Keypress(Key::Ctrl('r')) => Some(C::RedoLastEdit),
            Input::Keypress(Key::Ctrl('l')) => Some(C::ToggleLineNumbers),
            Input::Keypress(Key::Ctrl('f')) => Some(C::SwitchMode(M::Search)),
            Input::Keypress(Key::Ctrl('g')) => Some(C::SwitchMode(M::Goto)),
            Input::Keypress(Key::CtrlDown) => Some(C::ScrollViewportDown),
            Input::Keypress(Key::CtrlUp) => Some(C::ScrollViewportUp),
            Input::Keypress(Key::Char(ch)) => match self.cursor.get_highlighted_range() {
                None => Some(C::Edit {
                    clear_selection: false,
                    op: E::InsertCharAt {
                        pos: self.cursor.pos(),
                        ch,
                    },
                }),
                Some((pos_from, pos_to)) => Some(C::Edit {
                    clear_selection: true,
                    op: E::ReplaceRange {
                        pos_from,
                        pos_to,
                        string: String::from(ch),
                    },
                }),
            },
            Input::Keypress(Key::Backspace) | Input::Keypress(Key::Delete) => {
                match self.cursor.get_highlighted_range() {
                    None => Some(C::Edit {
                        clear_selection: false,
                        op: E::DeleteCharAt {
                            pos: self.cursor.pos(),
                        },
                    }),
                    Some((pos_from, pos_to)) => Some(C::Edit {
                        clear_selection: true,
                        op: E::DeleteRange { pos_from, pos_to },
                    }),
                }
            }
            Input::Keypress(Key::Ctrl('h')) | Input::Keypress(Key::Ctrl('\x7F')) => {
                match self.cursor.get_highlighted_range() {
                    None => Some(C::Edit {
                        clear_selection: false,
                        op: E::DeletePreviousWord {
                            pos: self.cursor.pos(),
                        },
                    }),
                    Some((pos_from, pos_to)) => Some(C::Edit {
                        clear_selection: true,
                        op: E::DeleteRange { pos_from, pos_to },
                    }),
                }
            }
            Input::Keypress(Key::Left) => Some(C::MoveCursor {
                with_selection: false,
                movement: MoveCursor::Left(1),
            }),
            Input::Keypress(Key::Right) => Some(C::MoveCursor {
                with_selection: false,
                movement: MoveCursor::Right(1),
            }),
            Input::Keypress(Key::Up) => Some(C::MoveCursor {
                with_selection: false,
                movement: MoveCursor::Up(1),
            }),
            Input::Keypress(Key::Down) => Some(C::MoveCursor {
                with_selection: false,
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
                    with_selection: false,
                    movement: MoveCursor::PreviousWord,
                })
            }
            Input::Keypress(Key::Alt('b')) | Input::Keypress(Key::CtrlLeft) => {
                Some(C::MoveCursor {
                    with_selection: false,
                    movement: MoveCursor::NextWord,
                })
            }
            _ => None,
        }
    }
}
