#[derive(Default)]
pub struct StatusBar {
    pub prompt_line: String,
    pub notification: String,
    pub cursor_pos: usize,

    pub has_displayed_unsaved_file_msg: bool,
}

impl StatusBar {
    pub fn update_notification(&mut self, msg: impl AsRef<str>) {
        self.notification.truncate(0);
        self.notification.push_str(msg.as_ref());
    }
}
