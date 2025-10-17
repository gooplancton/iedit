use std::io::{self, Read};
use termion::event::Key;
use termion::input::TermRead;

use crate::{
    editor::state::EditorMode,
    line::{CharacterEditable, EditorLine},
};

use super::{Editor, cursor::MovementDirection};

pub enum EditorInput {
    NoOp,
    CharInsertion(char),
    NewlineInsertion,
    TabInsertion,
    Deletion,
    WordDeletion,
    BasicMovement(MovementDirection),
    SelectionMovement(MovementDirection),
    WordMovement(MovementDirection),
    Save,
    Quit,
    ToggleLineNumbers,
    EnterCommandMode(&'static str),
    BackToDefaultMode,
}

pub trait InputReader {
    fn get_input(&mut self) -> io::Result<EditorInput>;
}

impl InputReader for io::Stdin {
    fn get_input(&mut self) -> io::Result<EditorInput> {
        use EditorInput::*;
        use MovementDirection::*;
        // Use termion's keys() iterator which properly handles all escape sequences
        if let Some(key) = io::stdin().keys().next() {
            match key? {
                // Arrow keys
                Key::Up => Ok(BasicMovement(Up)),
                Key::Down => Ok(BasicMovement(Down)),
                Key::Left => Ok(BasicMovement(Left)),
                Key::Right => Ok(BasicMovement(Right)),

                // Ctrl + Arrow keys for word movement
                Key::CtrlRight => Ok(WordMovement(Right)),
                Key::CtrlLeft => Ok(WordMovement(Left)),

                // Shift + Arrow keys for selection movement
                Key::ShiftUp => Ok(SelectionMovement(Up)),
                Key::ShiftDown => Ok(SelectionMovement(Down)),
                Key::ShiftLeft => Ok(SelectionMovement(Left)),
                Key::ShiftRight => Ok(SelectionMovement(Right)),

                // MacOS specific word movement keys
                Key::Alt('f') => Ok(WordMovement(Right)),
                Key::Alt('b') => Ok(WordMovement(Left)),

                // Control keys
                Key::Ctrl('q') => Ok(Quit),
                Key::Ctrl('s') => Ok(Save),
                Key::Ctrl('l') => Ok(ToggleLineNumbers),
                Key::Ctrl('g') => Ok(EnterCommandMode("goto ")),
                Key::Ctrl('f') => Ok(EnterCommandMode("find ")),

                // Ctrl + Backspace for deleting the previous word
                Key::Ctrl('\x7F') => Ok(WordDeletion),
                Key::Ctrl('h') => Ok(WordDeletion),

                // Backspace/Delete
                Key::Backspace | Key::Delete => Ok(Deletion),

                Key::Char('\n') | Key::Char('\r') => Ok(NewlineInsertion),
                Key::Char('\t') => Ok(TabInsertion),
                Key::Char(c) => Ok(CharInsertion(c)),

                Key::Esc => Ok(BackToDefaultMode),

                // TODO: support utf-8 chars

                // Ignore other keys
                _ => Ok(NoOp),
            }
        } else {
            // No input available
            Ok(EditorInput::NoOp)
        }
    }
}

impl<TextLine: EditorLine> Editor<TextLine> {
    pub fn process_input(&mut self, input: EditorInput) -> std::io::Result<()> {
        let prev_x = self.state.cursor_pos_x as isize;
        let prev_y = self.state.cursor_pos_y as isize;

        match input {
            EditorInput::CharInsertion(c) => {
                if c == 'n' && matches!(self.state.mode, EditorMode::Find(_)) {
                    self.goto_next_match();
                    return Ok(());
                }

                if self.state.selection_anchor.is_some() {
                    self.delete_selection();
                    self.state.selection_anchor = None;
                }
                self.insert_char(c)
            }
            EditorInput::NewlineInsertion => match self.state.mode {
                EditorMode::Insert => self.insert_newline(),
                EditorMode::Command => self.run_command(),
                EditorMode::Find(_) => {
                    self.state.command_text.truncate_chars(0);
                    self.state.mode = EditorMode::Insert;
                }
            },
            EditorInput::TabInsertion => self.insert_tab(),
            EditorInput::Deletion => {
                if self.state.selection_anchor.is_some() {
                    self.delete_selection();
                    self.state.selection_anchor = None;
                } else {
                    self.delete_char();
                }
            }
            EditorInput::WordDeletion => {
                if self.state.selection_anchor.is_some() {
                    self.delete_selection();
                    self.state.selection_anchor = None;
                } else {
                    self.delete_word();
                }
            }
            EditorInput::SelectionMovement(dir) => {
                if self.state.selection_anchor.is_none() {
                    self.state.selection_anchor =
                        Some((self.state.cursor_pos_x, self.state.cursor_pos_y));
                }
                match dir {
                    MovementDirection::Up => self.move_cursor_up(),
                    MovementDirection::Down => self.move_cursor_down(),
                    MovementDirection::Left => self.move_cursor_left(),
                    MovementDirection::Right => self.move_cursor_right(),
                }
            }
            EditorInput::BasicMovement(dir) => {
                self.state.selection_anchor = None;
                match dir {
                    MovementDirection::Up => self.move_cursor_up(),
                    MovementDirection::Down => self.move_cursor_down(),
                    MovementDirection::Left => self.move_cursor_left(),
                    MovementDirection::Right => self.move_cursor_right(),
                }
            }
            EditorInput::WordMovement(dir) => {
                self.state.selection_anchor = None;
                match dir {
                    MovementDirection::Up => self.move_cursor_up(),
                    MovementDirection::Down => self.move_cursor_down(),
                    MovementDirection::Left => self.move_cursor_word_left(),
                    MovementDirection::Right => self.move_cursor_word_right(),
                }
            }
            EditorInput::Save => {
                self.save_file()?;
            }
            EditorInput::Quit => {
                self.quit();
            }
            EditorInput::ToggleLineNumbers => {
                self.config.show_line_numbers = !self.config.show_line_numbers;
            }
            EditorInput::EnterCommandMode(prefix) => {
                if self.state.mode == EditorMode::Insert {
                    self.enter_command_mode(prefix);
                }
            }
            EditorInput::BackToDefaultMode => {
                self.state.command_text.truncate_chars(0);
                self.state.mode = EditorMode::Insert;
            }
            EditorInput::NoOp => {}
        }

        self.clamp_cursor();

        self.state.cursor_vel_x = self.state.cursor_pos_x as isize - prev_x;
        self.state.cursor_vel_y = self.state.cursor_pos_y as isize - prev_y;

        self.adjust_viewport();

        self.render()?;
        Ok(())
    }
}
