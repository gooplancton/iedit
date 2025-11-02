use termion::event::Key;

use crate::{
    Editor,
    editor::{
        commands::{CommandExecutionResult, EditorCommand},
        modes::EditorMode,
    },
    input::Input,
};

impl Editor {
    pub fn goto_mode_execute_command(
        &mut self,
        command: EditorCommand,
        original_pos: (usize, usize),
    ) -> CommandExecutionResult {
        use CommandExecutionResult as R;
        use EditorCommand as C;
        use EditorMode as M;

        match command {
            C::InsertCharPrompt { pos_x: _, ch: _ }
            | C::DeleteCharPrompt { pos_x: _ }
            | C::MovePromptCursorLeft
            | C::MovePromptCursorRight => {
                self.prompt_mode_execute_command(command);
                let maybe_parsed_line_num = str::parse::<usize>(&self.status_bar.prompt_line);
                if let Ok(line_num) = maybe_parsed_line_num
                    && line_num > 0
                {
                    self.cursor.update_pos((0, line_num - 1));
                }
                R::Continue
            }
            C::SubmitPrompt => {
                self.status_bar.prompt_line.truncate(0);
                self.mode = M::Insert;
                R::Continue
            }
            C::SwitchMode(mode) => {
                self.status_bar.prompt_line.truncate(0);
                self.cursor.update_pos(original_pos);
                self.mode = mode;
                R::Continue
            }
            _ => R::Continue,
        }
    }

    pub fn goto_mode_parse_command(&self, input: Input) -> Option<EditorCommand> {
        use EditorCommand as C;

        match input {
            Input::Keypress(Key::Char('\n')) | Input::Keypress(Key::Char('\r')) => {
                Some(C::SubmitPrompt)
            }
            // can implement go to end, etc..
            Input::Keypress(Key::Char(ch)) if ch.is_numeric() => Some(C::InsertCharPrompt {
                pos_x: self.status_bar.cursor_pos,
                ch,
            }),
            Input::Keypress(Key::Char(ch)) if !ch.is_numeric() => None,
            _ => self.prompt_mode_parse_command(input),
        }
    }
}
