use termion::event::Key;

use crate::{editor::state::EditorMode, line::EditorLine};

use super::{Editor, cursor::MovementDirection};

pub enum EditorInput {
    NoOp,
    CharInsertion(char),
    NewlineInsertion,
    TabInsertion,
    Deletion,
    WordDeletion,
    ScrollViewport(MovementDirection),
    BasicMovement(MovementDirection),
    SelectionMovement(MovementDirection),
    WordMovement(MovementDirection),
    PageDown,
    PageUp,
    Save,
    Quit,
    Undo,
    Redo,
    ToggleLineNumbers,
    EnterCommandMode(&'static str),
    BackToDefaultMode,
}

pub fn process_key_event(key: Key) -> EditorInput {
    use EditorInput::*;
    use MovementDirection::*;
    match key {
        // Arrow keys
        Key::Up => BasicMovement(Up),
        Key::Down => BasicMovement(Down),
        Key::Left => BasicMovement(Left),
        Key::Right => BasicMovement(Right),

        // Ctrl + Arrow keys for word movement
        Key::CtrlRight => WordMovement(Right),
        Key::CtrlLeft => WordMovement(Left),
        Key::CtrlDown => ScrollViewport(Down),
        Key::CtrlUp => ScrollViewport(Up),

        // Shift + Arrow keys for selection movement
        Key::ShiftUp => SelectionMovement(Up),
        Key::ShiftDown => SelectionMovement(Down),
        Key::ShiftLeft => SelectionMovement(Left),
        Key::ShiftRight => SelectionMovement(Right),

        // MacOS specific word movement keys
        Key::Alt('f') => WordMovement(Right),
        Key::Alt('b') => WordMovement(Left),

        // Control keys
        Key::Ctrl('q') => Quit,
        Key::Ctrl('s') => Save,
        Key::Ctrl('l') => ToggleLineNumbers,
        Key::Ctrl('g') => EnterCommandMode("goto "),
        Key::Ctrl('f') => EnterCommandMode("find "),
        Key::Ctrl('z') => Undo,
        Key::Ctrl('r') => Redo,

        // Page up/down
        Key::Ctrl('d') => PageDown,
        Key::Ctrl('u') => PageUp,

        // Ctrl + Backspace for deleting the previous word
        Key::Ctrl('\x7F') => WordDeletion,
        Key::Ctrl('h') => WordDeletion,

        // Backspace/Delete
        Key::Backspace | Key::Delete => Deletion,

        Key::Char('\n') | Key::Char('\r') => NewlineInsertion,
        Key::Char('\t') => TabInsertion,
        Key::Char(c) => CharInsertion(c),

        Key::Esc => BackToDefaultMode,

        _ => NoOp,
    }
}

impl<TextLine: EditorLine> Editor<TextLine> {
    pub fn process_input(&mut self, input: EditorInput) -> std::io::Result<()> {
        if !matches!(input, EditorInput::ScrollViewport(_)) {
            self.state.viewport.vertical_offset = 0;
        }

        match input {
            EditorInput::Undo => {
                self.undo_last_edit();
            }
            EditorInput::Redo => {
                self.redo_last_edit();
            }
            EditorInput::PageUp => {
                self.move_cursor_page_up();
            }
            EditorInput::PageDown => {
                self.move_cursor_page_down();
            }
            EditorInput::ScrollViewport(MovementDirection::Up) => {
                if self.state.viewport.top_line > 0 {
                    self.state.viewport.vertical_offset += -1;
                }
            }
            EditorInput::ScrollViewport(MovementDirection::Down) => {
                if self.state.viewport.top_line + (self.config.n_lines as usize)
                    < self.file_lines.len()
                {
                    self.state.viewport.vertical_offset += 1;
                }
            }
            EditorInput::CharInsertion(c) => {
                if self.state.mode == EditorMode::Command || self.state.mode == EditorMode::Insert {
                    if self.state.selection_anchor.is_some() {
                        self.delete_selection();
                        self.state.selection_anchor = None;
                    }
                    self.insert_char(c)
                } else if matches!(self.state.mode, EditorMode::Find(_)) {
                    match c {
                        'n' => self.goto_next_match(),
                        'b' => self.goto_previous_match(),
                        _ => {}
                    };
                }
            }
            EditorInput::NewlineInsertion => match self.state.mode {
                EditorMode::Insert => self.insert_newline(),
                EditorMode::Command => {
                    self.state.should_run_command = true;
                }
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
                self.quit()?;
            }
            EditorInput::ToggleLineNumbers => {
                self.config.show_line_numbers = !self.config.show_line_numbers;
                self.needs_full_rerender = true;
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
            _ => {}
        }

        Ok(())
    }
}
