use std::io::{self, Read};

use super::{Editor, cursor::MovementDirection};

pub enum EditorCommand {
    Save,
    Quit,
    ToggleLineNumbers,
    // StartSelection,
    // ExitSelection,
}

pub enum EditorInput {
    CharInsertion(char),
    CharDeletion,
    BasicMovement(MovementDirection),
    Command(EditorCommand),
}

pub trait InputReader: Read {
    fn get_input(&mut self) -> io::Result<EditorInput>;
}

impl InputReader for io::Stdin {
    fn get_input(&mut self) -> io::Result<EditorInput> {
        let mut buf = [0_u8; 1];
        self.read_exact(&mut buf)?;

        match buf[0] {
            b'\x1B' => {
                // Possible arrow key escape sequence: ESC [ A/B/C/D
                let mut seq = [0_u8; 2];
                // Read the next two bytes; this will succeed for typical arrow key sequences.
                self.read_exact(&mut seq)?;
                if seq[0] == b'[' {
                    let input = match seq[1] {
                        b'A' => EditorInput::BasicMovement(MovementDirection::Up),
                        b'B' => EditorInput::BasicMovement(MovementDirection::Down),
                        b'D' => EditorInput::BasicMovement(MovementDirection::Left),
                        b'C' => EditorInput::BasicMovement(MovementDirection::Right),
                        _ => EditorInput::CharInsertion('\x1B'),
                    };
                    Ok(input)
                } else {
                    // Not an arrow sequence; fall back to treating ESC as a char
                    Ok(EditorInput::CharInsertion('\x1B'))
                }
            }
            // Ctrl-Q (0x11) -> Quit
            b'\x11' => Ok(EditorInput::Command(EditorCommand::Quit)),
            // Ctrl-S (0x13) -> Save
            b'\x13' => Ok(EditorInput::Command(EditorCommand::Save)),
            // Ctrl-L (0x0C) -> Toggle Line Numbers
            b'\x0C' => Ok(EditorInput::Command(EditorCommand::ToggleLineNumbers)),
            // Backspace or Delete
            8 | 127 => Ok(EditorInput::CharDeletion),
            // Enter
            b'\n' | b'\r' => Ok(EditorInput::CharInsertion('\n')),
            c => Ok(EditorInput::CharInsertion(c as char)),
        } //  ////  //
    }
}

impl Editor {
    pub fn process_input(&mut self, input: EditorInput) -> std::io::Result<()> {
        let prev_x = self.state.cursor_pos_x as isize;
        let prev_y = self.state.cursor_pos_y as isize;

        match input {
            EditorInput::CharInsertion(c) => {
                self.insert_char(c);
            }
            EditorInput::CharDeletion => {
                self.delete_char();
            }
            EditorInput::BasicMovement(dir) => match dir {
                MovementDirection::Up => self.move_cursor_up(),
                MovementDirection::Down => self.move_cursor_down(),
                MovementDirection::Left => self.move_cursor_left(),
                MovementDirection::Right => self.move_cursor_right(),
            },
            EditorInput::Command(cmd) => match cmd {
                EditorCommand::Save => {
                    self.save_file()?;
                }
                EditorCommand::Quit => {
                    unreachable!()
                }
                EditorCommand::ToggleLineNumbers => {
                    self.config.show_line_numbers = !self.config.show_line_numbers;
                }
            },
        }

        self.clamp_cursor();

        self.state.cursor_vel_x = self.state.cursor_pos_x as isize - prev_x;
        self.state.cursor_vel_y = self.state.cursor_pos_y as isize - prev_y;

        self.adjust_viewport();

        self.render()?;
        Ok(())
    }
}

impl EditorInput {
    pub fn should_quit(&self) -> bool {
        matches!(self, EditorInput::Command(EditorCommand::Quit))
    }
}
