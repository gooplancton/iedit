pub struct EditorConfig {
    pub n_lines: u16,
    pub horizontal_margin: u16,
    pub vertical_margin: u16,
    pub show_line_numbers: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            n_lines: 0,
            horizontal_margin: 4,
            vertical_margin: 4,
            show_line_numbers: true
        }
    }
}

