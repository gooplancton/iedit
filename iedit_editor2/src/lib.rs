pub mod editor;
pub mod input;

#[macro_use]
mod terminal;

pub use editor::Editor;

pub mod line {
    pub use iedit_document::{CharacterEditable, DocumentLine};
}
