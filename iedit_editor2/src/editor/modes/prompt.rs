use iedit_document::EditOperation;
use termion::event::Key;

use crate::{
    editor::{commands::{CommandExecutionResult, EditorCommand}, modes::EditorMode}, input::Input, Editor
};

impl Editor {
    pub fn prompt_mode_execute_command(
        &mut self,
        command: EditorCommand,
    ) -> CommandExecutionResult {
        todo!()
    }

    pub fn prompt_mode_parse_command(&self, input: Input) -> Option<EditorCommand> {
        use EditOperation as E;
        use EditorCommand as C;
        use EditorMode as M;

        match input {
            Input::Keypress(Key::Esc) => Some(C::SwitchMode(M::Insert)),
            Input::Keypress(Key::Left) => Some(C::MovePromptCursorLeft),
            Input::Keypress(Key::Right) => Some(C::MovePromptCursorRight),
            Input::Keypress(Key::Backspace) | Input::Keypress(Key::Delete) => {
                Some(C::DeleteCharPrompt {
                    pos_x: self.status_bar.cursor_pos,
                })
            }
            Input::Keypress(Key::Char('\n')) | Input::Keypress(Key::Char('\r')) => {
                Some(C::SubmitPrompt)
            }
            Input::Keypress(Key::Char(ch)) => Some(C::InsertCharPrompt {
                pos_x: self.status_bar.cursor_pos,
                ch,
            }),
            _ => None,
        }
    }
}
