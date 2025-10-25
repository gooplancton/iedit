pub mod editor;

#[macro_use]
mod terminal;

pub use editor::Editor;

pub mod line {
    pub use iedit_document::{CharacterEditable, DocumentLine};
}
