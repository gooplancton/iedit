mod edit;
mod find;

use std::{
    fs::File,
    ops::{self},
    path::{Path, PathBuf},
};

pub use crate::line::{CharacterIndexable, DocumentLine};
pub use edit::{EditOperation, InverseStack, Text};

use crate::io::read_file;

pub struct Document {
    pub lines: Vec<DocumentLine>,
    pub file: Option<File>,
    pub canonicalized_file_path: PathBuf,
    undo_stack: Vec<EditOperation>,
    redo_stack: Vec<EditOperation>,

    pub has_been_edited: bool,
    pub is_readonly: bool,
}

impl Document {
    pub fn is_line_dirty(&self, line_idx: usize) -> bool {
        self.lines.get(line_idx).is_none_or(|line| line.is_dirty)
    }

    pub fn clean_lines(&mut self, line_range: ops::Range<usize>) {
        for line_idx in line_range {
            if let Some(line) = self.lines.get_mut(line_idx) {
                line.is_dirty = false;
            }
        }
    }

    pub fn from_strings(lines: Vec<String>, name: impl Into<PathBuf>, is_readonly: bool) -> Self {
        Self {
            lines: lines.into_iter().map(DocumentLine::new).collect(),
            file: None,
            canonicalized_file_path: name.into(),
            undo_stack: vec![],
            redo_stack: vec![],
            has_been_edited: false,
            is_readonly,
        }
    }

    pub fn from_file(file_path: impl AsRef<Path>) -> std::io::Result<Self> {
        let (file, canonicalized_file_path, lines) = read_file(file_path)?;
        let is_readonly = if let Some(file) = &file
            && let Ok(metadata) = file.metadata()
        {
            metadata.permissions().readonly()
        } else {
            false
        };

        Ok(Self {
            lines,
            file,
            canonicalized_file_path,
            undo_stack: vec![],
            redo_stack: vec![],
            has_been_edited: false,
            is_readonly,
        })
    }

    pub fn get_name(&self) -> Option<&str> {
        None
    }

    #[inline]
    pub fn n_lines(&self) -> usize {
        self.lines.len()
    }

    #[inline]
    pub fn get_or_add_line(&mut self, y: usize) -> Option<&mut DocumentLine> {
        if y < self.n_lines() {
            self.lines.get_mut(y)
        } else if y == self.n_lines() {
            self.lines.push(DocumentLine::default());
            self.lines.last_mut()
        } else {
            None
        }
    }

}
