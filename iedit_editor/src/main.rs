use std::io::stdout;

use iedit_editor::{Editor, config::EditorConfig, terminal::UILayout};
use termion::raw::IntoRawMode;

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let path = args.next();
    let open_at = args.next();

    match [path.as_deref(), open_at.as_deref()] {
        [Some("--version"), None] => {
            println!("iedit version {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        [Some("--help"), None] => {
            println!("Usage: iedit [FILE] [LINE_NUMBER]");
            println!();
            println!("Open FILE in the editor, optionally starting at LINE_NUMBER");
            println!();
            println!("Options:");
            println!("  --help     Show this help message");
            println!("  --version  Show version information");
            return Ok(());
        }
        [Some(path), open_at] => {
            let open_at = open_at
                .and_then(|open_at| open_at.parse::<usize>().ok())
                .unwrap_or_default();

            let editor_config = if let Some(mut path) = std::env::home_dir() {
                path.push(".iedit.conf");
                EditorConfig::from_file(path).unwrap_or_default()
            } else {
                EditorConfig::default()
            };

            let mut terminal = stdout().into_raw_mode()?;
            let ui = UILayout::new(editor_config.min_lines, &mut terminal)?;

            let mut editor = Editor::new(path, open_at, editor_config, ui)?;

            editor.run(&mut terminal)?;

            Ok(())
        }
        [None, _] => {
            let editor_config = if let Some(mut path) = std::env::home_dir() {
                path.push(".iedit.conf");
                EditorConfig::from_file(path).unwrap_or_default()
            } else {
                EditorConfig::default()
            };

            let mut terminal = stdout().into_raw_mode()?;
            let ui = UILayout::new(editor_config.min_lines, &mut terminal)?;

            let mut editor = Editor::new(String::new(), 0, editor_config, ui)?;

            editor.run(&mut terminal)?;

            Ok(())
        }
    }
}
