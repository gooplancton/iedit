use editor::Editor;

mod editor;
#[macro_use]
mod terminal;

fn main() {
    let mut editor = Editor::new("iedit.py").expect("file not found");
    editor.run().unwrap();
}
