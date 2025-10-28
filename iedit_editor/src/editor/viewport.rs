use std::cmp::{max, min};

use crate::editor::Editor;

#[derive(Default)]
pub struct Viewport {
    pub top_line: usize,
    pub left_col: usize,
    pub pre_scroll_top_line: usize,
    pub vertical_offset: isize,
}

impl Viewport {}

impl Editor {
    pub fn adjust_viewport(&mut self) {
        let n_lines = self.config.n_lines as usize;
        let term_width = self.renderer.term_width as usize;
        let n_cols = term_width - self.config.show_line_numbers as usize * 7;
        let y = self.cursor.cur_y;
        let past_y = self.cursor.past_y;
        let x = self.cursor.cur_x;
        let past_x = self.cursor.past_x;
        let is_cursor_visible = self.viewport_contains_y(y);

        // If cursor moved and is not visible, reset vertical offset and scroll to cursor
        // FIXME: this won't work if the user submits an input that results in the cursor not moving
        // e.g. if the cursor is past the viewport at col. 0, moving left will not center the viewport
        if (y != past_y || x != past_x) && !is_cursor_visible {
            self.viewport.vertical_offset = 0;
            self.viewport.pre_scroll_top_line = y.saturating_sub(n_lines / 2);
            self.renderer.needs_full_rerender = true;
        }
        // maintain user-adjusted vertical offset if cursor is still visible
        else if self.viewport.vertical_offset != 0 && is_cursor_visible {
            if self.viewport.vertical_offset >= 0 {
                self.viewport.pre_scroll_top_line += self.viewport.vertical_offset as usize;
            } else {
                self.viewport.pre_scroll_top_line = self
                    .viewport
                    .pre_scroll_top_line
                    .saturating_sub(-self.viewport.vertical_offset as usize);
            }
            self.viewport.vertical_offset = 0;
            self.renderer.needs_full_rerender = true;
        }

        let vertical_margin = self.config.vertical_margin as usize;
        let horizontal_margin = self.config.horizontal_margin as usize;

        let top_limit = self.viewport.pre_scroll_top_line + vertical_margin;
        let bottom_limit = self.viewport.pre_scroll_top_line + n_lines - vertical_margin;

        let should_scroll_up = y < top_limit && y < past_y;
        let should_scroll_down = y > bottom_limit && y > past_y;

        let lines_below_viewport = self
            .document
            .n_lines()
            .saturating_sub(self.viewport.top_line + n_lines - 1);

        if should_scroll_up {
            let vertical_scroll = top_limit.saturating_sub(y);
            self.viewport.pre_scroll_top_line = self
                .viewport
                .pre_scroll_top_line
                .saturating_sub(vertical_scroll);
            self.renderer.needs_full_rerender = true;
        } else if should_scroll_down && lines_below_viewport > 0 {
            let vertical_scroll = min(lines_below_viewport, y.saturating_sub(bottom_limit));
            self.viewport.pre_scroll_top_line += vertical_scroll;
            self.renderer.needs_full_rerender = true;
        }

        self.viewport.top_line = self.viewport.pre_scroll_top_line;

        let offset = self.viewport.vertical_offset;
        if offset != 0 {
            self.renderer.needs_full_rerender = true;
        }
        self.viewport.top_line = min(
            self.document.n_lines(),
            max(0, (self.viewport.top_line as isize) + offset) as usize,
        );

        let left_limit = self.viewport.left_col + horizontal_margin;
        let right_limit = self.viewport.left_col + term_width - vertical_margin;

        if x < left_limit && x < past_x {
            let horizontal_scroll = left_limit.saturating_sub(x);
            self.viewport.left_col = self.viewport.left_col.saturating_sub(horizontal_scroll);
            self.renderer.needs_full_rerender = true;
        } else if x > right_limit && x > past_x {
            let horizontal_scroll = x.saturating_sub(right_limit);
            self.viewport.left_col = (self.viewport.left_col + term_width).saturating_sub(n_cols);
            self.renderer.needs_full_rerender = true;
        }
    }

    #[inline(always)]
    pub fn viewport_contains_y(&self, y: usize) -> bool {
        self.viewport.top_line <= y && y < self.viewport.top_line + self.config.n_lines as usize
    }
}
