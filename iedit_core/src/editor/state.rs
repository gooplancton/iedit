use std::cmp::{max, min};

use crate::editor::{EditorLine, viewport::Viewport};

use super::Editor;

#[derive(Default)]
pub struct EditorState {
    pub status_text: String,
    pub viewport: Viewport,
    pub cursor_pos_x: usize,
    pub cursor_pos_y: usize,
    pub ideal_cursor_pos_x: usize,
    pub cursor_vel_x: isize,
    pub cursor_vel_y: isize,
    pub selection_anchor: Option<(usize, usize)>,

    pub is_file_modified: bool,
}

impl<TextLine: EditorLine> Editor<TextLine> {
    pub fn get_current_line(&self) -> &TextLine {
        &self.file_lines[self.state.cursor_pos_y]
    }

    pub fn update_status_text(&mut self) {
        let line = self.state.cursor_pos_y + 1;
        let col = self.state.cursor_pos_x + 1;
        let modified = if self.state.is_file_modified { "*" } else { "" };
        let total_lines = self.file_lines.len();
        self.state.status_text = format!(
            "{}{} | Ln {}, Col {} | {} lines",
            self.file_name, modified, line, col, total_lines
        )
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
