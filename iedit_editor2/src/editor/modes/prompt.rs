use crate::{
    Editor,
    editor::commands::{CommandExecutionResult, EditorCommand},
    input::Input,
};

impl Editor {
    pub fn prompt_mode_execute_command(
        &mut self,
        command: &EditorCommand,
    ) -> CommandExecutionResult {
        todo!()
    }

    pub fn prompt_mode_parse_command(&self, input: Input) -> Option<EditorCommand> {
        todo!()
    }
}
