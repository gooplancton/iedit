mod document;
mod io;
mod line;

pub use document::{
    CharacterIndexable, Document, DocumentSyntax, EditOperation, InverseStack, SyntaxBlock,
    SyntaxRule, Text,
};
pub use line::DocumentLine;
