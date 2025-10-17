use crate::{
    editor::{EditorLine, viewport::Viewport},
    line::CharacterEditable,
};

use super::Editor;

#[derive(PartialEq)]
pub enum EditorMode {
    Insert,
    Command,
    Find((usize, usize)),
}

impl Default for EditorMode {
    fn default() -> Self {
        Self::Insert
    }
}

#[derive(Default)]
pub struct EditorState<TextLine: EditorLine> {
    pub status_text: TextLine,
    pub command_text: TextLine,
    pub viewport: Viewport,
    pub cursor_pos_x: usize,
    pub cursor_pos_y: usize,
    pub ideal_cursor_pos_x: usize,
    pub cmd_cursor_pos_x: usize,
    pub cursor_vel_x: isize,
    pub cursor_vel_y: isize,
    pub selection_anchor: Option<(usize, usize)>,

    pub mode: EditorMode,
    pub is_file_modified: bool,
    pub should_quit: bool,
    pub should_run_command: bool,
}

impl<TextLine: EditorLine> EditorState<TextLine> {
    pub fn get_cursor_pos(&mut self) -> (usize, Option<usize>) {
        match self.mode {
            EditorMode::Insert => (self.cursor_pos_x, Some(self.cursor_pos_y)),
            _ => (self.cmd_cursor_pos_x, None),
        }
    }

    pub fn get_cursor_pos_mut(&mut self) -> (&mut usize, Option<&mut usize>) {
        match self.mode {
            EditorMode::Insert => (&mut self.cursor_pos_x, Some(&mut self.cursor_pos_y)),
            _ => (&mut self.cmd_cursor_pos_x, None),
        }
    }

    pub fn set_ideal_cursor_pos_x(&mut self) {
        if self.mode == EditorMode::Insert {
            self.ideal_cursor_pos_x = self.cursor_pos_x;
        }
    }

    pub fn is_editing_content(&self) -> bool {
        matches!(self.mode, EditorMode::Insert)
    }
}

impl<TextLine: EditorLine> Editor<TextLine> {
    pub fn get_current_line(&self) -> &TextLine {
        match self.state.mode {
            EditorMode::Insert => &self.file_lines[self.state.cursor_pos_y],
            _ => &self.state.command_text,
        }
    }

    pub fn get_current_line_mut(&mut self) -> &mut TextLine {
        match self.state.mode {
            EditorMode::Insert => &mut self.file_lines[self.state.cursor_pos_y],
            _ => &mut self.state.command_text,
        }
    }

    pub fn update_status_text(&mut self) {
        let line = self.state.cursor_pos_y + 1;
        let col = self.state.cursor_pos_x + 1;
        let modified = if self.state.is_file_modified { "*" } else { "" };
        let total_lines = self.file_lines.len();
        let file_name = self
            .canonicalized_file_path
            .components()
            .next_back()
            .map(|f| f.as_os_str())
            .unwrap_or_default();
        self.state.status_text = TextLine::from_str(&format!(
            "{:?}{} | Ln {}, Col {} | {} lines",
            file_name, modified, line, col, total_lines
        ))
    }

    pub fn goto_next_match(&mut self) {
        if let Some((x, y)) = self.find_next_match() {
            let term_len = self.get_search_term().len();
            self.state.cursor_pos_x = x;
            self.state.cursor_pos_y = y;
            self.state.selection_anchor = Some((x + term_len, y));
            self.state.set_ideal_cursor_pos_x();
            self.state.mode = EditorMode::Find((x + 1, y))
        }
    }

    pub fn get_highlighted_range(&self) -> ((usize, usize), (usize, usize)) {
        let cursor_pos = (self.state.cursor_pos_x, self.state.cursor_pos_y);
        match self.state.selection_anchor {
            Some(anchor_pos) => {
                if anchor_pos.1 < cursor_pos.1
                    || (anchor_pos.1 == cursor_pos.1 && anchor_pos.0 < cursor_pos.0)
                {
                    (anchor_pos, cursor_pos)
                } else {
                    (cursor_pos, anchor_pos)
                }
            }
            None => {
                let mut anchor_pos = cursor_pos;
                anchor_pos.0 += 1;
                (cursor_pos, anchor_pos)
            }
        }
    }
}
