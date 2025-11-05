use std::cmp::min;

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
    pub fn prompt_mode_execute_command(
        &mut self,
        command: EditorCommand,
    ) -> CommandExecutionResult {
        use CommandExecutionResult as R;
        use EditorCommand as C;

        match command {
            C::MovePromptCursorLeft => {
                self.status_bar.cursor_pos = self.status_bar.cursor_pos.saturating_sub(1);

                R::Continue
            }
            C::MovePromptCursorRight => {
                self.status_bar.cursor_pos = min(
                    self.status_bar.prompt_line.len(),
                    self.status_bar.cursor_pos + 1,
                );

                R::Continue
            }
            C::InsertCharPrompt { pos_x, ch } => {
                self.status_bar.prompt_line.insert(pos_x, ch);
                self.status_bar.cursor_pos = pos_x + 1;

                R::Continue
            }
            C::DeleteCharPrompt { pos_x } => {
                if pos_x == 0 {
                    return R::Continue;
                }

                self.status_bar.prompt_line.remove(pos_x - 1);
                self.status_bar.cursor_pos = pos_x - 1;

                R::Continue
            }
            C::SwitchMode(mode) => {
                self.mode = mode;

                R::Continue
            }
            C::SubmitPrompt => {
                let prompt = self.status_bar.prompt_line.split_off(0);
                if let Some(fun) = self.status_bar.submit_action.take() {
                    let res = fun(self, prompt);
                    if !matches!(res, R::Continue) {
                        return res;
                    }
                }

                self.mode = EditorMode::Insert;

                R::Continue
            }
            _ => R::Continue,
        }
    }

    pub fn prompt_mode_parse_command(&self, input: Input) -> Option<EditorCommand> {
        use EditorCommand as C;
        use EditorMode as M;

        match input {
            Input::Keypress(Key::Esc) => Some(C::SwitchMode(M::Insert)),
            Input::Keypress(Key::Left) => Some(C::MovePromptCursorLeft),
            Input::Keypress(Key::Right) => Some(C::MovePromptCursorRight),
            Input::Keypress(Key::Backspace) | Input::Keypress(Key::Delete) => {
                Some(C::DeleteCharPrompt {
                    pos_x: self.status_bar.cursor_pos,
                })
            }
            Input::Keypress(Key::Char('\n')) | Input::Keypress(Key::Char('\r')) => {
                Some(C::SubmitPrompt)
            }
            Input::Keypress(Key::Char(ch)) => Some(C::InsertCharPrompt {
                pos_x: self.status_bar.cursor_pos,
                ch,
            }),
            _ => None,
        }
    }

    pub fn prompt_user(
        &mut self,
        prompt: &'static str,
        callback: impl FnOnce(&mut Editor, String) -> CommandExecutionResult + 'static,
    ) {
        self.mode = EditorMode::Prompt(prompt);
        self.status_bar.submit_action = Some(Box::from(callback));
    }
}
