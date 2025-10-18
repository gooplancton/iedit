use std::io;

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

impl<TextLine: EditorLine> EditorState<TextLine> {
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

type ModifyStateFn<TextLine> = dyn Fn(&mut EditorState<TextLine>);

impl<TextLine: EditorLine> Editor<TextLine> {
    pub fn run_command(&mut self) -> io::Result<()> {
        let command = self.state.parse_command();

        let modify_state: Option<&ModifyStateFn<TextLine>> = match command {
            EditorCommand::GotoLine(idx) => {
                self.goto_line(idx);
                Some(&move |state: &mut EditorState<TextLine>| {
                    state.command_text.truncate_chars(0);
                    state.mode = EditorMode::default();
                })
            }
            EditorCommand::Write => {
                self.save_file()?;
                Some(&move |state: &mut EditorState<TextLine>| {
                    state.command_text.truncate_chars(0);
                    state.mode = EditorMode::default();
                })
            }
            EditorCommand::WriteAndQuit => {
                self.save_file()?;
                self.quit()?;
                None
            }
            EditorCommand::Quit => {
                self.quit()?;
                None
            }
            EditorCommand::Find => {
                if let Some((x, y)) = self.find_next_match() {
                    let term_len = self.get_search_term().len();
                    Some(&move |state: &mut EditorState<TextLine>| {
                        state.cursor_pos_x = x;
                        state.cursor_pos_y = y;
                        state.ideal_cursor_pos_x = x;
                        state.selection_anchor = Some((x + term_len, y));
                        state.set_ideal_cursor_pos_x();
                        state.mode = EditorMode::Find((x + 1, y));
                    })
                } else {
                    None
                }
            }
            EditorCommand::NoOp => None,
        };

        if let Some(modify_state) = modify_state {
            modify_state(&mut self.state);
        }

        Ok(())
    }

    pub fn enter_command_mode(&mut self, prefix: &str) {
        self.state.selection_anchor = None;
        self.state.mode = EditorMode::Command;
        self.state.command_text.push_str(prefix);
        self.state.cmd_cursor_pos_x = self.state.command_text.len();
    }

    pub fn get_search_term(&self) -> &TextLine::SliceType {
        self.state.command_text.get_chars(5..)
    }

    pub fn find_next_match(&self) -> Option<(usize, usize)> {
        let (start_x, start_y) = if let EditorMode::Find((x, y)) = self.state.mode {
            (x, y)
        } else {
            (0, 0)
        };

        let term = self.get_search_term();

        for (y, line) in self.file_lines.iter().enumerate().skip(start_y) {
            let offset = start_x * (y == start_y) as usize;
            if let Some(x) = line.get_chars(offset..).find_term_from(term, offset) {
                return Some((x + offset, y));
            }
        }

        None
    }
}
