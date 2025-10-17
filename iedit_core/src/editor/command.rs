use crate::{Editor, line::EditorLine};

pub enum EditorCommand {
    GotoLine(usize),
    Invalid,
}

impl<TextLine: EditorLine> Editor<TextLine> {
    pub fn parse_command(&mut self) -> EditorCommand {
        match self
            .state
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
                    EditorCommand::Invalid
                }
            }
            _ => EditorCommand::Invalid,
        }
    }

    pub fn run_command(&mut self) {
        let command = self.parse_command();

        match command {
            EditorCommand::GotoLine(idx) => {
                self.goto_line(idx);
            }
            EditorCommand::Invalid => {}
        };

        self.state.command_text.truncate_chars(0);
        self.state.is_entering_command = false;
    }

    pub fn enter_command_mode(&mut self, prefix: &str) {
        self.state.is_entering_command = true;
        self.state.command_text.push_str(prefix);
        self.state.cmd_cursor_pos_x = self.state.command_text.len();
    }

    pub fn goto_line(&mut self, idx: usize) {
        self.state.cursor_pos_y = idx.saturating_sub(1);
    }
}
