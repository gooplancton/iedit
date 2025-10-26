use crate::{
    Editor,
    editor::commands::{CommandExecutionResult, EditorCommand},
    input::Input,
};

mod goto;
mod insert;
mod prompt;
mod search;

pub enum EditorMode {
    Insert,
    Prompt,
    Goto,
    Search,
}

impl Editor {
    #[inline]
    pub fn execute_command(&mut self, command: &EditorCommand) -> CommandExecutionResult {
        match self.mode {
            EditorMode::Insert => self.insert_mode_execute_command(command),
            EditorMode::Prompt => self.prompt_mode_execute_command(command),
            EditorMode::Goto => self.goto_mode_execute_command(command),
            EditorMode::Search => self.search_mode_execute_command(command),
        }
    }

    #[inline]
    pub fn parse_command(&self, input: Input) -> Option<EditorCommand> {
        match self.mode {
            EditorMode::Insert => self.insert_mode_parse_command(input),
            EditorMode::Prompt => self.prompt_mode_parse_command(input),
            EditorMode::Goto => self.goto_mode_parse_command(input),
            EditorMode::Search => self.search_mode_parse_command(input),
        }
    }
}
