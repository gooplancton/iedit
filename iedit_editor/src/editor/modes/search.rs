use std::str::FromStr;

use regex_lite::Regex;
use termion::event::Key;

use crate::{
    editor::{
        commands::{send_notification, CommandExecutionResult, EditorCommand},
        search::SearchItem,
    }, input::Input, Editor
};

impl Editor {
    pub fn search_mode_execute_command(
        &mut self,
        command: EditorCommand,
        original_pos: (usize, usize),
        is_backwards: bool,
    ) -> CommandExecutionResult {
        use CommandExecutionResult as R;
        use EditorCommand as C;

        match command {
            C::SubmitPrompt => {
                let maybe_parsed_regex = Regex::from_str(&self.status_bar.prompt_line);
                let next_cursor_pos = match (&maybe_parsed_regex, is_backwards) {
                    (Ok(regex), false) => {
                        self.document.get_next_regex_match_pos(original_pos, regex)
                    }
                    (Ok(regex), true) => self
                        .document
                        .get_previous_regex_match_pos(original_pos, regex),
                    (Err(_), false) => self
                        .document
                        .get_next_literal_match_pos(original_pos, &self.status_bar.prompt_line),
                    (Err(_), true) => self
                        .document
                        .get_previous_literal_match_pos(original_pos, &self.status_bar.prompt_line),
                };

                if let Some(next_cursor_pos) = next_cursor_pos {
                    self.cursor.update_pos(next_cursor_pos);
                    if let Ok(regex) = maybe_parsed_regex {
                        self.search_item = Some(SearchItem::Regex(regex))
                    } else {
                        self.search_item = Some(SearchItem::PromptString)
                    }
                } else {
                    send_notification("No matches");
                }

                R::Continue
            }
            C::FindMatchForward => {
                let next_cursor_pos = match &self.search_item {
                    Some(SearchItem::Regex(regex)) => {
                        self.document.get_next_regex_match_pos(original_pos, regex)
                    }
                    Some(SearchItem::PromptString) => self
                        .document
                        .get_next_literal_match_pos(original_pos, &self.status_bar.prompt_line),
                    _ => return R::Continue,
                };

                if let Some(next_cursor_pos) = next_cursor_pos {
                    self.cursor.update_pos(next_cursor_pos);
                }

                R::Continue
            }
            C::FindMatchBackward => {
                let next_cursor_pos = match &self.search_item {
                    Some(SearchItem::Regex(regex)) => self
                        .document
                        .get_previous_regex_match_pos(original_pos, regex),
                    Some(SearchItem::PromptString) => self
                        .document
                        .get_previous_literal_match_pos(original_pos, &self.status_bar.prompt_line),
                    _ => return R::Continue,
                };

                if let Some(next_cursor_pos) = next_cursor_pos {
                    self.cursor.update_pos(next_cursor_pos);
                }

                R::Continue
            }
            C::InsertCharPrompt { pos_x: _, ch: _ }
            | C::DeleteCharPrompt { pos_x: _ }
            | C::MovePromptCursorLeft
            | C::MovePromptCursorRight => self.prompt_mode_execute_command(command),
            _ => self.goto_mode_execute_command(command, original_pos),
        }
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
