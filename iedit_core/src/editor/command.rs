use crate::{Editor, editor::state::EditorState, line::EditorLine};

pub enum EditorCommand<'a> {
    GotoLine(usize),
    Find(&'a str),
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
            ["find", term] => EditorCommand::Find(term),
            ["w"] | ["write"] => EditorCommand::Write,
            ["q"] | ["quit"] => EditorCommand::Quit,
            ["wq"] => EditorCommand::WriteAndQuit,
            _ => EditorCommand::NoOp,
        }
    }
}

impl<TextLine: EditorLine> Editor<TextLine> {
    pub fn run_command(&mut self) {
        let command = self.state.parse_command();

        let modify_state = match command {
            EditorCommand::GotoLine(idx) => {
                self.goto_line(idx);
                None
            }
            EditorCommand::WriteAndQuit => {
                self.save_file();
                self.quit();
                None
            }
            EditorCommand::Write => {
                self.save_file();
                None
            }
            EditorCommand::Quit => {
                self.quit();
                None
            }
            EditorCommand::Find(term) => {
                if let Some((x, y)) = self.find_term(term) {
                    let term_len = term.len();
                    Some(move |state: &mut EditorState<TextLine>| {
                        state.cursor_pos_x = x;
                        state.cursor_pos_y = y;
                        state.selection_anchor = Some((x + term_len, y));
                        state.set_ideal_cursor_pos_x();
                    })
                } else {
                    None
                }
            }
            EditorCommand::NoOp => None,
        };

        self.state.is_entering_command = false;
        self.state.command_text.truncate_chars(0);

        if let Some(modify_state) = modify_state {
            modify_state(&mut self.state);
        }
    }

    pub fn enter_command_mode(&mut self, prefix: &str) {
        if self.state.is_entering_command {
            return;
        }

        self.state.selection_anchor = None;
        self.state.is_entering_command = true;
        self.state.command_text.push_str(prefix);
        self.state.cmd_cursor_pos_x = self.state.command_text.len();
    }

    pub fn find_term(&self, term: &str) -> Option<(usize, usize)> {
        for (y, line) in self.file_lines.iter().enumerate() {
            if let Some(x) = line.find_term(term) {
                return Some((x, y));
            }
        }

        None
    }
}
