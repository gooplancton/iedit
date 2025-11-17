use iedit_macros::ConfigParse;

#[derive(ConfigParse)]
pub struct EditorConfig {
    pub fullscreen: bool,
    pub min_lines: u16,
    pub horizontal_margin: u16,
    pub vertical_margin: u16,
    pub tab_size: u16,
    pub show_line_numbers: bool,
    pub show_keybindings: bool,
    pub confirm_quit_unsaved_changes: bool,
    pub tab_emit_spaces: bool,
    pub enable_syntax_highlighting: bool,
    pub syntax_highlighting_dir: Option<String>,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            fullscreen: false,
            min_lines: 0,
            horizontal_margin: 4,
            tab_size: 4,
            vertical_margin: 4,
            show_line_numbers: true,
            show_keybindings: true,
            confirm_quit_unsaved_changes: true,
            tab_emit_spaces: true,
            enable_syntax_highlighting: true,
            syntax_highlighting_dir: None,
        }
    }
}
