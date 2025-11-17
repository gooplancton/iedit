use std::io::{self, IsTerminal, Read, stdin, stdout};

use iedit_document::Document;
use iedit_editor::{Editor, config::EditorConfig, terminal::UILayout};
use termion::raw::IntoRawMode;

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let path = args.next();
    let open_at = args.next();

    let editor_config_path = std::env::home_dir().map(|mut home_path| {
        home_path.push(".iedit.conf");
        home_path
    });
    let editor_config = if let Some(editor_config_path) = &editor_config_path {
        EditorConfig::from_file(editor_config_path).unwrap_or_default()
    } else {
        EditorConfig::default()
    };

    let mut terminal = stdout().into_raw_mode()?;
    let ui = if editor_config.fullscreen {
        UILayout::fullscreen(&mut terminal)
    } else {
        UILayout::new(editor_config.min_lines, &mut terminal)
    }?;

    let mut editor = match [path.as_deref(), open_at.as_deref()] {
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
        [Some("--config"), None] => {
            let document = if let Some(editor_config_path) = &editor_config_path {
                Document::from_file(editor_config_path)?
            } else {
                return Err(io::Error::other("Could not determine config path"));
            };

            Editor::new(document, 0, editor_config, ui)?
        }
        [Some(path), open_at] => {
            let open_at = open_at
                .and_then(|open_at| open_at.parse::<usize>().ok())
                .unwrap_or_default();

            let document = Document::from_file(path)?;

            Editor::new(document, open_at, editor_config, ui)?
        }
        [None, _] => {
            let document = if !stdin().is_terminal() {
                let mut buffer = String::new();
                stdin().read_to_string(&mut buffer)?;
                let lines: Vec<String> = buffer.lines().map(|s| s.to_string()).collect();
                Document::from_strings(lines, "", false)
            } else {
                Document::default()
            };

            Editor::new(document, 0, editor_config, ui)?
        }
    };

    loop {
        let result = editor.run(&mut terminal)?;

        match result {
            iedit_editor::editor::EditorRunResult::RestartInFullscreenMode => {
                let new_ui = UILayout::fullscreen(&mut terminal)?;
                editor.set_ui(new_ui);
            }
            iedit_editor::editor::EditorRunResult::Quit => break,
        };
    }

    Ok(())
}
