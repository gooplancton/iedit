use std::{io, str::FromStr};

use regex_lite::Regex;

use crate::{
    Editor,
    editor::state::{EditorMode, EditorState},
    line::{CharacterEditable, EditorLine},
};

pub enum EditorCommand {
    GotoLine(usize),
    Find,
    WriteAndQuit,
    Write,
    Quit,
    NoOp,
}

impl EditorState {
    pub fn parse_command(&self) -> EditorCommand {
        match self
            .command_text
            .as_str()
            .split(' ')
            .map(|chunk| chunk.trim())
            .collect::<Vec<&str>>()
            .as_slice()
        {
            ["goto", idx] => {
                if let Ok(idx) = idx.parse::<usize>() {
                    EditorCommand::GotoLine(idx)
                } else {
                    EditorCommand::NoOp
                }
            }
            ["find", ..] => EditorCommand::Find,
            ["w"] | ["write"] => EditorCommand::Write,
            ["q"] | ["quit"] => EditorCommand::Quit,
            ["wq"] => EditorCommand::WriteAndQuit,
            _ => EditorCommand::NoOp,
        }
    }
}

impl Editor {
    pub fn run_command(&mut self) -> io::Result<()> {
        let command = self.state.parse_command();

        match command {
            EditorCommand::GotoLine(idx) => {
                self.goto_line(idx);
                self.state.command_text.truncate_chars(0);
                self.state.mode = EditorMode::default();
            }
            EditorCommand::Write => {
                self.save_file()?;
                self.state.command_text.truncate_chars(0);
                self.state.mode = EditorMode::default();
            }
            EditorCommand::WriteAndQuit => {
                self.save_file()?;
                self.quit()?;
            }
            EditorCommand::Quit => {
                self.quit()?;
            }
            EditorCommand::Find => {
                self.state.searched_regex = Regex::from_str(self.get_search_input_str()).ok();
                self.goto_next_match();
            }
            EditorCommand::NoOp => {}
        };

        Ok(())
    }

    pub fn enter_command_mode(&mut self, prefix: &str) {
        self.state.selection_anchor = None;
        self.state.mode = EditorMode::Command;
        self.state.command_text.push_str(prefix);
        self.state.cmd_cursor_pos_x = self.state.command_text.len();
    }

    pub fn get_search_input_str(&self) -> &str {
        self.state.command_text.get_chars(5..)
    }

    pub fn goto_next_match(&mut self) {
        let (start_x, start_y) = if let EditorMode::Find((x, y)) = self.state.mode {
            (x + 1, y)
        } else {
            (0, 0)
        };

        if let Some(regex) = &self.state.searched_regex {
            for (y, line) in self.file_lines.iter().enumerate().skip(start_y) {
                let offset = start_x * (y == start_y) as usize;
                if let Some((x_start, x_end)) = line.find_regex_from(regex, offset) {
                    self.state.cursor_pos_x = x_start;
                    self.state.cursor_pos_y = y;
                    self.state.ideal_cursor_pos_x = x_start;
                    self.state.selection_anchor = Some((x_end, y));
                    self.state.mode = EditorMode::Find((x_start, y));
                    return;
                }
            }
        }

        self.temp_message = "Already at last match".to_owned();
    }

    pub fn goto_previous_match(&mut self) {
        let (end_x, end_y) = if let EditorMode::Find((x, y)) = self.state.mode {
            (x.saturating_sub(1), y)
        } else {
            (0, 0)
        };

        if let Some(regex) = &self.state.searched_regex {
            for (y, line) in self.file_lines.iter().enumerate().take(end_y + 1).rev() {
                let line = if y == end_y {
                    line.get_chars(..end_x)
                } else {
                    line.get_chars(..)
                };
                if let Some((x_start, x_end)) = line.find_last_match(regex) {
                    self.state.cursor_pos_x = x_start;
                    self.state.cursor_pos_y = y;
                    self.state.ideal_cursor_pos_x = x_start;
                    self.state.selection_anchor = Some((x_end, y));
                    self.state.mode = EditorMode::Find((x_start, y));
                    return;
                }
            }
        }

        self.temp_message = "Already at first match".to_owned();
    }
}
