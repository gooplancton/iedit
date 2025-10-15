use iedit_core::{Editor, line::DefaultLineType};

fn main() -> std::io::Result<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or(std::io::Error::other("no file given"))?;

    let mut editor = Editor::<DefaultLineType>::new(path)?;

    editor.run()?;

    Ok(())
}
