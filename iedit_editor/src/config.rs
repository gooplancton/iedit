use iedit_document::{DocumentSyntax, SyntaxRule};
use iedit_macros::{ConfigParse, Reflective};
use regex_lite::Regex;

#[derive(ConfigParse, Reflective)]
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
    pub use_system_clipboard: bool,
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
            use_system_clipboard: true,
            confirm_quit_unsaved_changes: true,
            tab_emit_spaces: true,
            enable_syntax_highlighting: true,
            syntax_highlighting_dir: None,
        }
    }
}

pub fn editor_config_syntax() -> DocumentSyntax {
    let regex = format!("^({})", EditorConfig::field_names().join("|"));

    DocumentSyntax {
        name: "iedit",
        rules: vec![SyntaxRule::Inline {
            color: termion::color::Green.fg_str().to_owned(),
            pattern: Regex::new(&regex).unwrap(),
        }],
    }
}
