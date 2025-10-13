use iedit_core::Editor;

fn main() -> std::io::Result<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or(std::io::Error::other("no file given"))?;

    let mut editor = Editor::new(path)?;

    editor.run()?;

    Ok(())
}
