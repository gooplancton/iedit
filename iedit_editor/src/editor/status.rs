use iedit_document::DocumentLine;

use crate::{Editor, editor::commands::CommandExecutionResult};

type SubmitAction = Box<dyn FnOnce(&mut Editor, DocumentLine) -> CommandExecutionResult>;

pub static KEYBINDINGS: &str = "│ ^q: quit │ ^s: save │ ^t: help";

#[derive(Default)]
pub struct StatusBar {
    pub prompt_line: DocumentLine,
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

pub static FLAGS: [&str; 4] = [
    "\x1b[30;103m modified \x1b[0m",
    "\x1b[30;104m sel. lock \x1b[0m",
    "\x1b[30;101m running cmd \x1b[0m",
    "\x1b[30;102m cmd output \x1b[0m",
];

pub static FLAGS_SMALL: [&str; 4] = [
    "\x1b[30;103m * \x1b[0m",
    "\x1b[30;104m sel \x1b[0m",
    "\x1b[30;101m cmd \x1b[0m",
    "\x1b[30;102m out \x1b[0m",
];

impl Editor {
    pub fn get_flag_strings(&self) -> impl Iterator<Item = &str> {
        let small = self.ui.term_width < 80;

        [
            self.document.has_been_modified(),
            self.is_selection_locked,
            self.is_running_external_command,
            self.is_viewing_execution_output,
        ]
        .into_iter()
        .enumerate()
        .filter(|(_, flag)| *flag)
        .map(move |(idx, _)| if small { FLAGS_SMALL[idx] } else { FLAGS[idx] })
    }
}
