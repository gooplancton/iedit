use std::io;

use editor::Editor;

mod editor;
#[macro_use]
mod terminal;

fn main() -> io::Result<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or(io::Error::other("no file given"))?;

    let mut editor = Editor::new(path)?;

    editor.run()?;

    Ok(())
}
