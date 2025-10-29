use termion::event::Key;

use crate::{
    Editor,
    editor::commands::{CommandExecutionResult, EditorCommand},
    input::Input,
};

mod goto;
mod insert;
mod prompt;
mod search;

type OriginalCursorPos = (usize, usize);

pub enum EditorMode {
    Insert,
    Prompt(&'static str),
    Goto(OriginalCursorPos),
    Search,
}

static UNSAVED_CHANGES_WARNING: &str =
    "\x1b[33mBuffer contains unsaved changes.\x1b[0m Ctrl+ \x1b[7ms\x1b[0mave, \x1b[7mq\x1b[0muit";

impl Editor {
    pub fn execute_command(&mut self, command: EditorCommand) -> CommandExecutionResult {
        use EditorCommand as C;

        match command {
            C::DisplayExternalNotification(notification) => {
                self.status_bar.notification = notification;
                CommandExecutionResult::Continue
            }
            C::Quit => self.quit(false),
            C::Save => {
                if let Err(err) = self.save_file() {
                    self.status_bar.update_notification(err.to_string());
                };
                CommandExecutionResult::Continue
            }
            C::ToggleLockSelection => {
                self.is_selection_locked = !self.is_selection_locked;
                CommandExecutionResult::Continue
            }
            C::ToggleLineNumbers => {
                self.config.show_line_numbers = !self.config.show_line_numbers;
                CommandExecutionResult::Continue
            }
            C::ScrollViewportUp => {
                if self.viewport.top_line > 0 {
                    self.viewport.vertical_offset -= 1;
                }
                CommandExecutionResult::Continue
            }
            C::ScrollViewportDown => {
                if self.viewport.top_line + (self.config.n_lines as usize) < self.document.n_lines()
                {
                    self.viewport.vertical_offset += 1;
                }
                CommandExecutionResult::Continue
            }
            _ => match self.mode {
                EditorMode::Insert => self.insert_mode_execute_command(command),
                EditorMode::Prompt(_) => self.prompt_mode_execute_command(command),
                EditorMode::Goto(original_pos) => {
                    self.goto_mode_execute_command(command, original_pos)
                }
                EditorMode::Search => self.search_mode_execute_command(command),
            },
        }
    }

    #[inline]
    pub fn parse_command(&self, input: Input) -> Option<EditorCommand> {
        match input {
            Input::ExternalNotification(notification) => {
                Some(EditorCommand::DisplayExternalNotification(notification))
            }
            Input::Keypress(Key::Ctrl('q')) => Some(EditorCommand::Quit),
            Input::Keypress(Key::Ctrl('s')) => Some(EditorCommand::Save),
            _ => match self.mode {
                EditorMode::Insert => self.insert_mode_parse_command(input),
                EditorMode::Prompt(_) => self.prompt_mode_parse_command(input),
                EditorMode::Goto(_) => self.goto_mode_parse_command(input),
                EditorMode::Search => self.search_mode_parse_command(input),
            },
        }
    }

    pub fn quit(&mut self, force: bool) -> CommandExecutionResult {
        if !self.document.has_been_edited
            || !self.config.confirm_quit_unsaved_changes
            || self.first_quit_sent
            || force
        {
            CommandExecutionResult::ShouldQuit
        } else {
            self.status_bar.update_notification(UNSAVED_CHANGES_WARNING);
            self.first_quit_sent = true;
            CommandExecutionResult::Continue
        }
    }
}
