mod edit;
mod find;

use std::{
    fs::File,
    ops::{self},
    path::{Path, PathBuf},
    time::SystemTime,
};

pub use crate::line::{CharacterIndexable, DocumentLine};
pub use edit::{EditOperation, InverseStack, Text};

use crate::io::read_file;

pub struct Document {
    pub lines: Vec<DocumentLine>,
    pub file: Option<File>,
    pub canonicalized_file_path: PathBuf,
    pub line_offsets: Vec<u64>,
    undo_stack: Vec<EditOperation>,
    redo_stack: Vec<EditOperation>,

    pub end_of_line_seq: String,
    pub last_save_time: SystemTime,
    pub is_readonly: bool,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            lines: Default::default(),
            file: Default::default(),
            canonicalized_file_path: Default::default(),
            line_offsets: Default::default(),
            undo_stack: Default::default(),
            redo_stack: Default::default(),
            end_of_line_seq: "\n".to_owned(),
            last_save_time: SystemTime::now(),
            is_readonly: false,
        }
    }
}

impl Document {
    pub fn line_needs_render(&self, line_idx: usize) -> bool {
        self.lines
            .get(line_idx)
            .is_none_or(|line| line.needs_render)
    }

    pub fn reset_lines_need_render(&mut self, line_range: ops::Range<usize>) {
        for line_idx in line_range {
            if let Some(line) = self.lines.get_mut(line_idx) {
                line.needs_render = false;
            }
        }
    }

    pub fn from_strings(strings: Vec<String>, name: impl Into<PathBuf>, is_readonly: bool) -> Self {
        Self {
            lines: strings.into_iter().map(DocumentLine::new).collect(),
            file: None,
            canonicalized_file_path: name.into(),
            line_offsets: vec![],
            undo_stack: vec![],
            redo_stack: vec![],
            end_of_line_seq: "\n".to_owned(),
            last_save_time: SystemTime::now(),
            is_readonly,
        }
    }

    pub fn from_file(file_path: impl AsRef<Path>) -> std::io::Result<Self> {
        let (file, canonicalized_file_path, lines, line_offsets, end_of_line_seq) =
            read_file(file_path)?;
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
            end_of_line_seq,
            line_offsets,
            undo_stack: vec![],
            redo_stack: vec![],
            last_save_time: SystemTime::now(),
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
    pub fn has_been_modified(&self) -> bool {
        self.lines.iter().any(|line| line.has_been_modified)
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
