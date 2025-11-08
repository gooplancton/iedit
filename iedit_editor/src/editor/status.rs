use iedit_document::DocumentLine;

use crate::{Editor, editor::commands::CommandExecutionResult};

type SubmitAction = Box<dyn FnOnce(&mut Editor, DocumentLine) -> CommandExecutionResult>;

static KEYBINDINGS: &str = " ^q: quit; ^s: save; ^t: help";

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

impl Editor {
    pub fn get_status_text_chunks(&self) -> [&str; 3] {
        [
            self.document
                .canonicalized_file_path
                .as_os_str()
                .to_str()
                .and_then(|file_name| {
                    if file_name.is_empty() {
                        None
                    } else {
                        Some(file_name)
                    }
                })
                .unwrap_or("[Unnamed Buffer]"),
            if self.document.has_been_edited {
                "* |"
            } else {
                " |"
            },
            if self.config.show_keybindings {
                KEYBINDINGS
            } else {
                ""
            },
        ]
    }
}
