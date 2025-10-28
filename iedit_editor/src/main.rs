use iedit_editor::Editor;

fn main() -> std::io::Result<()> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_default();

    let open_at = std::env::args()
        .nth(2)
        .and_then(|open_at| open_at.parse::<usize>().ok())
        .unwrap_or_default();

    let mut editor = Editor::new(path, open_at)?;

    editor.run()?;

    Ok(())
}
