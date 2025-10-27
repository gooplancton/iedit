use iedit_document::EditOperation;
use termion::event::Key;

use crate::{
    Editor,
    editor::{
        commands::{CommandExecutionResult, EditorCommand},
        modes::EditorMode,
    },
    input::Input,
};

impl Editor {
    pub fn goto_mode_execute_command(&mut self, command: EditorCommand) -> CommandExecutionResult {
        todo!()
    }

    pub fn goto_mode_parse_command(&self, input: Input) -> Option<EditorCommand> {
        use EditOperation as E;
        use EditorCommand as C;
        use EditorMode as M;

        match input {
            Input::Keypress(Key::Char('\n')) | Input::Keypress(Key::Char('\r')) => {
                Some(C::SubmitPrompt)
            }
            // can implement go to end, etc..
            Input::Keypress(Key::Char(ch)) if ch.is_numeric() => Some(C::InsertCharPrompt {
                pos_x: self.status_bar.cursor_pos,
                ch,
            }),
            Input::Keypress(Key::Char(ch)) if !ch.is_numeric() => None,
            _ => self.prompt_mode_parse_command(input),
        }
    }
}
