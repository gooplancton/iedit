use editor::Editor;

mod editor;
#[macro_use]
mod terminal;

fn main() {
    let path = std::env::args().nth(1).expect("no file given");
    let mut editor = Editor::new(path).expect("file not found");

    editor.run().unwrap();
}
