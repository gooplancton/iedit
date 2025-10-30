pub mod editor;
pub mod input;
pub mod config;

#[macro_use]
pub mod terminal;

pub use editor::Editor;

pub mod line {
    pub use iedit_document::{CharacterEditable, DocumentLine};
}
