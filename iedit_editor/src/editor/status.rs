use crate::{Editor, editor::commands::CommandExecutionResult};

type SubmitAction = Box<dyn FnOnce(&mut Editor, String) -> CommandExecutionResult>;

#[derive(Default)]
pub struct StatusBar {
    pub prompt_line: String,
    pub notification: String,
    pub cursor_pos: usize,

    pub submit_action: Option<SubmitAction>,
}

impl StatusBar {
    pub fn update_notification(&mut self, msg: impl AsRef<str>) {
        self.notification.truncate(0);
        self.notification.push_str(msg.as_ref());
    }
}
