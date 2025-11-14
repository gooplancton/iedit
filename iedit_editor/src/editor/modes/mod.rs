use termion::event::Key;

use crate::{
    Editor,
    editor::{
        commands::{CommandExecutionResult, EditorCommand, send_simple_notification},
        keybindings::{
            CHORDS_POPUP_LINES, HELP_POPUP_LINES, L_CHORD_POPUP_LINES, S_CHORD_POPUP_LINES,
            T_CHORD_POPUP_LINES, V_CHORD_POPUP_LINES, X_CHORD_POPUP_LINES,
        },
    },
    input::{Input, Notification},
};

mod goto;
mod insert;
mod prompt;
mod search;

pub enum EditorMode {
    Insert,
    Prompt(&'static str),
    Goto {
        original_cursor_pos: (usize, usize),
    },
    Search {
        original_cursor_pos: (usize, usize),
        is_backwards: bool,
    },
}

static UNSAVED_CHANGES_WARNING: &str =
    "\x1b[33mBuffer contains unsaved changes.\x1b[0m Ctrl-s: save, Ctrl-q: quit";

impl Editor {
    pub fn execute_command(&mut self, command: EditorCommand) -> CommandExecutionResult {
        use EditorCommand as C;

        match command {
            C::MoveCursor {
                movement: _,
                with_selection: _,
            } => self.execute_cursor_movement_command(command),
            C::EndFileExecution(status, is_output_available) => {
                self.status_bar.notification = format!(
                    "{}. {}",
                    status,
                    if is_output_available {
                        "Ctrl-k + v + o: view output"
                    } else {
                        "output unavailable"
                    }
                );
                self.is_running_external_command = false;
            }
            C::DisplayHelp => {
                self.displayed_popup = Some(&HELP_POPUP_LINES);
            }
            C::DisplayChordsHelp => {
                self.displayed_popup = Some(&CHORDS_POPUP_LINES);
            }
            C::DisplayLineChordHelp => {
                self.displayed_popup = Some(&L_CHORD_POPUP_LINES);
            }
            C::DisplayExecuteChordHelp => {
                self.displayed_popup = Some(&X_CHORD_POPUP_LINES);
            }
            C::DisplayViewChordHelp => {
                self.displayed_popup = Some(&V_CHORD_POPUP_LINES);
            }
            C::DisplaySelectionChordHelp => {
                self.displayed_popup = Some(&S_CHORD_POPUP_LINES);
            }
            C::DisplayPressCharacterPopup => {
                self.displayed_popup = Some(&T_CHORD_POPUP_LINES);
            }
            C::DisplayMessage(notification) => {
                self.status_bar.notification = notification;
            }
            C::Quit => return self.quit(false),
            C::Save => {
                if let Err(err) = self.save_file(true) {
                    send_simple_notification(err.to_string());
                };
            }
            C::ToggleLockSelection => {
                self.needs_full_rerender = true;
                self.is_selection_locked = !self.is_selection_locked;
            }
            C::ToggleLineNumbers => {
                self.needs_full_rerender = true;
                self.config.show_line_numbers = !self.config.show_line_numbers;
            }
            C::ScrollViewportUp => {
                if self.viewport.top_line > 0 {
                    self.viewport.vertical_offset -= 1;
                }
            }
            C::ScrollViewportDown => {
                if self.viewport.top_line + (self.ui.editor_lines as usize)
                    < self.document.n_lines()
                {
                    self.viewport.vertical_offset += 1;
                }
            }
            _ => match self.mode {
                EditorMode::Insert => return self.insert_mode_execute_command(command),
                EditorMode::Prompt(_) => return self.prompt_mode_execute_command(command),
                EditorMode::Goto {
                    original_cursor_pos,
                } => return self.goto_mode_execute_command(command, original_cursor_pos),
                EditorMode::Search {
                    original_cursor_pos,
                    is_backwards,
                } => {
                    return self.search_mode_execute_command(
                        command,
                        original_cursor_pos,
                        is_backwards,
                    );
                }
            },
        };

        CommandExecutionResult::Continue
    }

    #[inline]
    pub fn parse_command(&self, input: Input) -> Option<EditorCommand> {
        match input {
            Input::ExternalNotification(Notification::Simple(message)) => {
                Some(EditorCommand::DisplayMessage(message))
            }
            Input::ExternalNotification(Notification::ExecutionEnd {
                status,
                output_available,
            }) => Some(EditorCommand::EndFileExecution(status, output_available)),
            Input::Keypress(Key::Ctrl('q')) => Some(EditorCommand::Quit),
            Input::Keypress(Key::Ctrl('s')) => Some(EditorCommand::Save),
            _ => match self.mode {
                EditorMode::Insert => self.insert_mode_parse_command(input),
                EditorMode::Prompt(_) => self.prompt_mode_parse_command(input),
                EditorMode::Goto {
                    original_cursor_pos: _,
                } => self.goto_mode_parse_command(input),
                EditorMode::Search {
                    original_cursor_pos: _,
                    is_backwards: _,
                } => self.search_mode_parse_command(input),
            },
        }
    }

    pub fn quit(&mut self, force: bool) -> CommandExecutionResult {
        if !self.document.has_been_modified()
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
