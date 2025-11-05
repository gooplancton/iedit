use std::cmp::min;

use crate::Editor;

pub struct Cursor {
    pub cur_x: usize,
    pub cur_y: usize,
    pub ideal_x: usize,
    pub past_x: usize,
    pub past_y: usize,
    pub selection_anchor: Option<(usize, usize)>,
}

impl Cursor {
    #[inline]
    pub fn update_pos(&mut self, new_pos: (usize, usize)) {
        let (new_x, new_y) = new_pos;

        self.cur_x = new_x;
        self.cur_y = new_y;
        self.ideal_x = new_x;
    }

    #[inline]
    pub fn set_last_pos(&mut self) {
        self.past_x = self.cur_x;
        self.past_y = self.cur_y;
    }

    #[inline]
    pub fn move_down(&mut self, lines: usize) {
        self.update_pos((self.ideal_x, self.cur_y.saturating_add(lines)));
    }

    #[inline]
    pub fn move_up(&mut self, lines: usize) {
        self.update_pos((self.ideal_x, self.cur_y.saturating_sub(lines)));
    }

    #[inline]
    pub fn move_right(&mut self, cols: usize) {
        self.update_pos((self.cur_x.saturating_add(cols), self.cur_y));
    }

    #[inline]
    pub fn move_left(&mut self, cols: usize) {
        self.update_pos((self.cur_x.saturating_sub(cols), self.cur_y));
    }

    #[inline]
    pub fn new(starting_pos: (usize, usize)) -> Self {
        let (x, y) = starting_pos;

        Self {
            cur_x: x,
            cur_y: y,
            ideal_x: x,
            past_x: x,
            past_y: y,
            selection_anchor: None,
        }
    }

    #[inline]
    pub const fn pos(&self) -> (usize, usize) {
        (self.cur_x, self.cur_y)
    }

    pub fn get_highlighted_range(&self) -> Option<((usize, usize), (usize, usize))> {
        let cursor_pos = self.pos();

        self.selection_anchor.map(|anchor_pos| {
            if anchor_pos.1 < cursor_pos.1
                || (anchor_pos.1 == cursor_pos.1 && anchor_pos.0 < cursor_pos.0)
            {
                (anchor_pos, cursor_pos)
            } else {
                (cursor_pos, anchor_pos)
            }
        })
    }
}

impl Editor {
    pub fn clamp_cursor(&mut self) {
        let max_x = self
            .document
            .lines
            .get(self.cursor.cur_y)
            .map_or(0, |line| line.len());

        self.cursor.cur_x = min(self.cursor.ideal_x, max_x);
        self.cursor.cur_y = min(self.cursor.cur_y, self.document.n_lines());
    }
}
