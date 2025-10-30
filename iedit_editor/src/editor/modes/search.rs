use termion::event::Key;

use crate::{
    Editor,
    editor::
        commands::{CommandExecutionResult, EditorCommand}
    ,
    input::Input,
};

impl Editor {
    pub fn search_mode_execute_command(
        &mut self,
        _command: EditorCommand,
    ) -> CommandExecutionResult {
        todo!()
    }

    pub fn search_mode_parse_command(&self, input: Input) -> Option<EditorCommand> {
        use EditorCommand as C;

        match input {
            Input::Keypress(Key::Ctrl('n')) => Some(C::FindMatchForward),
            Input::Keypress(Key::Ctrl('b')) => Some(C::FindMatchBackward),
            _ => self.prompt_mode_parse_command(input),
        }
    }
}
