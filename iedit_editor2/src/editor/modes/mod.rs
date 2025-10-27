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

pub enum EditorMode {
    Insert,
    Prompt,
    Goto,
    Search,
}

static UNSAVED_CHANGES_WARNING: &str = "\x1b[33mBuffer contains unsaved changes.\x1b[0m Ctrl+ \x1b[7ms\x1b[0mave; Ctrl\x1b[7mq\x1b0muit";

impl Editor {
    #[inline]
    pub fn execute_command(&mut self, command: EditorCommand) -> CommandExecutionResult {
        use EditorCommand as C;

        if !matches!(command, C::ScrollViewportUp) && !matches!(command, C::ScrollViewportDown) {
            if (self.viewport.top_line <= self.cursor.cur_y)
                && (self.cursor.cur_y < self.viewport.top_line + self.config.n_lines as usize)
            {
                self.viewport.pre_scroll_top_line = self.viewport.top_line;
            }
            self.viewport.vertical_offset = 0;
            self.renderer.needs_full_rerender = true;
        }

        match command {
            C::SwitchMode(editor_mode) => {
                self.mode = editor_mode;
                CommandExecutionResult::Continue
            }
            C::Quit => {
                if !self.document.has_been_edited || self.status_bar.has_displayed_unsaved_file_msg
                {
                    CommandExecutionResult::ShouldQuit
                } else {
                    self.status_bar.update_notification(UNSAVED_CHANGES_WARNING);
                    self.status_bar.has_displayed_unsaved_file_msg = true;
                    CommandExecutionResult::Continue
                }
            }
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
                EditorMode::Prompt => self.prompt_mode_execute_command(command),
                EditorMode::Goto => self.goto_mode_execute_command(command),
                EditorMode::Search => self.search_mode_execute_command(command),
            },
        }
    }

    #[inline]
    pub fn parse_command(&self, input: Input) -> Option<EditorCommand> {
        match input {
            Input::Keypress(Key::Ctrl('q')) => Some(EditorCommand::Quit),
            Input::Keypress(Key::Ctrl('s')) => Some(EditorCommand::Save),
            _ => match self.mode {
                EditorMode::Insert => self.insert_mode_parse_command(input),
                EditorMode::Prompt => self.prompt_mode_parse_command(input),
                EditorMode::Goto => self.goto_mode_parse_command(input),
                EditorMode::Search => self.search_mode_parse_command(input),
            },
        }
    }
}
