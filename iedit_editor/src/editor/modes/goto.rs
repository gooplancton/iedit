use termion::event::Key;

use crate::{
    Editor,
    editor::{
        commands::{CommandExecutionResult, CursorMovement, EditorCommand},
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
                let maybe_parsed_line_num =
                    str::parse::<usize>(self.status_bar.prompt_line.as_ref());
                if let Ok(line_num) = maybe_parsed_line_num
                    && line_num > 0
                {
                    self.cursor.update_pos((0, line_num - 1), false);
                    self.needs_full_rerender = true;
                }
                R::Continue
            }
            C::SubmitPrompt => {
                self.status_bar.prompt_line.truncate(0);
                self.mode = M::Insert;
                self.needs_full_rerender = true;
                self.cursor.jump_history.push(original_pos);
                R::Continue
            }
            C::SwitchMode(mode) => {
                self.status_bar.prompt_line.truncate(0);
                self.cursor.update_pos(original_pos, false);
                self.mode = mode;
                self.search_item = None;
                self.needs_full_rerender = true;
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
            Input::Keypress(Key::Char('w')) => Some(C::MoveCursor {
                movement: CursorMovement::StartOfFile,
                with_selection: self.is_selection_locked,
            }),
            Input::Keypress(Key::Char('e')) => Some(C::MoveCursor {
                movement: CursorMovement::EndOfFile,
                with_selection: self.is_selection_locked,
            }),
            Input::Keypress(Key::Char(ch)) if ch.is_numeric() => Some(C::InsertCharPrompt {
                pos_x: self.status_bar.cursor_pos,
                ch,
            }),
            Input::Keypress(Key::Char(ch)) if !ch.is_numeric() => None,
            _ => self.prompt_mode_parse_command(input),
        }
    }
}
