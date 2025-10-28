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
    pub fn search_mode_execute_command(
        &mut self,
        command: EditorCommand,
    ) -> CommandExecutionResult {
        todo!()
    }

    pub fn search_mode_parse_command(&self, input: Input) -> Option<EditorCommand> {
        use EditOperation as E;
        use EditorCommand as C;
        use EditorMode as M;

        match input {
            Input::Keypress(Key::Ctrl('n')) => Some(C::FindMatchForward),
            Input::Keypress(Key::Ctrl('b')) => Some(C::FindMatchBackward),
            _ => self.prompt_mode_parse_command(input),
        }
    }
}
