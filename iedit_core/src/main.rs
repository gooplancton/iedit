use iedit_core::{Editor, line::DefaultLineType};

fn main() -> std::io::Result<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or(std::io::Error::other("no file given"))?;

    let open_at = std::env::args()
        .nth(2)
        .and_then(|open_at| open_at.parse::<usize>().ok())
        .unwrap_or_default();

    let mut editor = Editor::<DefaultLineType>::new(path, open_at)?;

    editor.run()?;

    Ok(())
}
