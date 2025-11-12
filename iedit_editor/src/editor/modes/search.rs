use std::str::FromStr;

use regex_lite::Regex;
use termion::event::Key;

use crate::{
    Editor,
    editor::{
        commands::{CommandExecutionResult, EditorCommand},
        modes::EditorMode,
        search::SearchItem,
    },
    input::Input,
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
                self.search_item = None;
                self.status_bar.prompt_line.truncate(0);
                self.mode = EditorMode::Insert;
                self.needs_full_rerender = true;
                self.cursor.jump_history.push(original_pos);
                R::Continue
            }
            C::MovePromptCursorLeft | C::MovePromptCursorRight => {
                self.prompt_mode_execute_command(command)
            }
            C::InsertCharPrompt { pos_x: _, ch: _ } | C::DeleteCharPrompt { pos_x: _ } => {
                self.prompt_mode_execute_command(command);

                let maybe_parsed_regex = Regex::from_str(self.status_bar.prompt_line.as_ref());
                let next_cursor_pos = match (&maybe_parsed_regex, is_backwards) {
                    (Ok(regex), false) => {
                        self.document.get_next_regex_match_pos(original_pos, regex)
                    }
                    (Ok(regex), true) => self
                        .document
                        .get_previous_regex_match_pos(original_pos, regex),
                    (Err(_), false) => self.document.get_next_literal_match_pos(
                        original_pos,
                        self.status_bar.prompt_line.as_ref(),
                    ),
                    (Err(_), true) => self.document.get_previous_literal_match_pos(
                        original_pos,
                        self.status_bar.prompt_line.as_ref(),
                    ),
                };

                if let Some((start, end)) = next_cursor_pos {
                    self.needs_full_rerender = true;
                    self.cursor.update_pos(start, false);
                    self.matched_range = Some((start, end));
                    if let Ok(regex) = maybe_parsed_regex {
                        self.search_item = Some(SearchItem::Regex(regex))
                    } else {
                        self.search_item = Some(SearchItem::PromptString)
                    }
                }

                R::Continue
            }
            C::FindMatchForward => {
                let next_cursor_pos = match &self.search_item {
                    Some(SearchItem::Regex(regex)) => self
                        .document
                        .get_next_regex_match_pos(self.cursor.pos(), regex),
                    Some(SearchItem::PromptString) => self.document.get_next_literal_match_pos(
                        self.cursor.pos(),
                        self.status_bar.prompt_line.as_ref(),
                    ),
                    _ => return R::Continue,
                };

                if let Some((start, end)) = next_cursor_pos {
                    self.needs_full_rerender = true;
                    self.cursor.update_pos(start, false);
                    self.matched_range = Some((start, end));
                }

                R::Continue
            }
            C::FindMatchBackward => {
                let next_cursor_pos = match &self.search_item {
                    Some(SearchItem::Regex(regex)) => self
                        .document
                        .get_previous_regex_match_pos(self.cursor.pos(), regex),
                    Some(SearchItem::PromptString) => self.document.get_previous_literal_match_pos(
                        self.cursor.pos(),
                        self.status_bar.prompt_line.as_ref(),
                    ),
                    _ => return R::Continue,
                };

                if let Some((start, end)) = next_cursor_pos {
                    self.needs_full_rerender = true;
                    self.cursor.update_pos(start, false);
                    self.matched_range = Some((start, end));
                }

                R::Continue
            }
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
