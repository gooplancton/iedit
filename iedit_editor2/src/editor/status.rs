#[derive(Default)]
pub struct StatusBar {
    pub prompt_line: String,
    pub notification: Option<String>,
    pub cursor_pos: usize,
}

