use std::cmp::{max, min};

use crate::editor::{EditorLine, viewport::Viewport};

use super::Editor;

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

    pub is_file_modified: bool,
    pub is_entering_command: bool,
}

impl<TextLine: EditorLine> EditorState<TextLine> {
    pub fn get_cursor_pos(&mut self) -> (usize, Option<usize>) {
        if self.is_entering_command {
            (self.cmd_cursor_pos_x, None)
        } else {
            (self.cursor_pos_x, Some(self.cursor_pos_y))
        }
    }

    pub fn get_cursor_pos_mut(&mut self) -> (&mut usize, Option<&mut usize>) {
        if self.is_entering_command {
            (&mut self.cmd_cursor_pos_x, None)
        } else {
            (&mut self.cursor_pos_x, Some(&mut self.cursor_pos_y))
        }
    }

    pub fn set_ideal_cursor_pos_x(&mut self) {
        if !self.is_entering_command {
            self.ideal_cursor_pos_x = self.cursor_pos_x;
        }
    }
}

impl<TextLine: EditorLine> Editor<TextLine> {
    pub fn get_current_line(&self) -> &TextLine {
        if self.state.is_entering_command {
            &self.state.command_text
        } else {
            &self.file_lines[self.state.cursor_pos_y]
        }
    }

    pub fn get_current_line_mut(&mut self) -> &mut TextLine {
        if self.state.is_entering_command {
            &mut self.state.command_text
        } else {
            &mut self.file_lines[self.state.cursor_pos_y]
        }
    }

    pub fn update_status_text(&mut self) {
        let line = self.state.cursor_pos_y + 1;
        let col = self.state.cursor_pos_x + 1;
        let modified = if self.state.is_file_modified { "*" } else { "" };
        let total_lines = self.file_lines.len();
        self.state.status_text = TextLine::from_str(&format!(
            "{}{} | Ln {}, Col {} | {} lines",
            self.file_name, modified, line, col, total_lines
        ))
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
