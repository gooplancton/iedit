use std::io::stdout;

use iedit_editor::{Editor, config::EditorConfig, terminal::UILayout};
use termion::{cursor::HideCursor, raw::IntoRawMode};

fn main() -> std::io::Result<()> {
    let path = std::env::args().nth(1).unwrap_or_default();

    let open_at = std::env::args()
        .nth(2)
        .and_then(|open_at| open_at.parse::<usize>().ok())
        .unwrap_or_default();

    let editor_config = if let Some(mut path) = std::env::home_dir() {
        path.push(".iedit.conf");
        EditorConfig::from_file(path).unwrap_or_default()
    } else {
        EditorConfig::default()
    };

    let mut terminal = HideCursor::from(stdout().into_raw_mode()?);
    let ui = UILayout::new(editor_config.min_lines, &mut terminal)?;

    let mut editor = Editor::new(path, open_at, editor_config, ui)?;

    editor.run(&mut terminal)?;

    Ok(())
}
