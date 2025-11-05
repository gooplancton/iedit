use std::io::Write;

use crate::{editor::highlight::SelectionHighlight, terminal::EMPTY_CURSOR};
use termion::color;

pub struct ColorRange {
    start: usize,
    /// (inclusive)
    end: usize,
    is_bg: bool,
    color_str: &'static str,
}

pub struct LineRenderer<'line, 'writer, Writer: Write> {
    pub line: &'line str,
    pub display_range: (usize, usize),
    pub writer: &'writer mut Writer,
    pub color_ranges: Vec<ColorRange>,
    pub tab_size: usize,
    pub cursor_at_end: bool,
}

impl<'line, 'writer, Writer: Write> LineRenderer<'line, 'writer, Writer> {
    pub fn new(
        line: &'line str,
        display_range: (usize, usize),
        writer: &'writer mut Writer,
        tab_size: usize,
    ) -> Self {
        Self {
            line,
            display_range,
            color_ranges: vec![],
            writer,
            tab_size,
            cursor_at_end: false,
        }
    }

    fn render_line_char(&mut self, idx: usize, ch: char) -> std::io::Result<()> {
        self.color_ranges
            .iter()
            .filter(|range| {
                if idx == self.display_range.0 {
                    range.start <= idx
                } else {
                    range.start == idx
                }
            })
            .try_for_each(|range| write!(self.writer, "{}", range.color_str))?;

        if ch == '\t' {
            // FIXME: need to handle tabs properly with respect to display range
            let n_spaces = self.tab_size - (idx % self.tab_size);
            let tab_string = " ".repeat(n_spaces);
            write!(self.writer, "{}", &tab_string)?;
        } else {
            write!(self.writer, "{}", ch)?;
        }

        self.color_ranges
            .iter()
            .filter(|range| range.end == idx)
            .try_for_each(|range| {
                write!(
                    self.writer,
                    "{}",
                    if range.is_bg {
                        color::Reset.bg_str()
                    } else {
                        color::Reset.fg_str()
                    }
                )
            })?;

        Ok(())
    }

    pub fn add_selection_highlight(&mut self, selection_highlight: SelectionHighlight) {
        match selection_highlight {
            SelectionHighlight::None => {}
            SelectionHighlight::After(start) => {
                self.color_ranges.push(ColorRange {
                    start,
                    end: self.line.len(),
                    is_bg: true,
                    color_str: color::LightBlue.bg_str(),
                });
            }
            SelectionHighlight::Before(end) => {
                self.color_ranges.push(ColorRange {
                    start: 0,
                    end,
                    is_bg: true,
                    color_str: color::LightBlue.bg_str(),
                });
            }
            SelectionHighlight::WholeLine => {
                self.color_ranges.push(ColorRange {
                    start: 0,
                    end: self.line.len(),
                    is_bg: true,
                    color_str: color::LightBlue.bg_str(),
                });
            }
            SelectionHighlight::Range(start, end) => {
                self.color_ranges.push(ColorRange {
                    start,
                    end,
                    is_bg: true,
                    color_str: color::LightBlue.bg_str(),
                });
            }
        }
    }

    pub fn add_cursor(&mut self, cursor_x: usize) {
        if cursor_x >= self.line.len() {
            self.cursor_at_end = true;
            return;
        }

        let range_to_split_idx = self
            .color_ranges
            .iter()
            .position(|color_range| cursor_x >= color_range.start && cursor_x <= color_range.end);

        if let Some(range_to_split_idx) = range_to_split_idx {
            let range_to_split = self.color_ranges.remove(range_to_split_idx);
            if range_to_split.start != cursor_x || range_to_split.end != cursor_x {
                let left_range = ColorRange {
                    start: range_to_split.start,
                    end: cursor_x.saturating_sub(1),
                    is_bg: range_to_split.is_bg,
                    color_str: range_to_split.color_str,
                };
                let right_range = ColorRange {
                    start: cursor_x + 1,
                    end: range_to_split.end,
                    is_bg: range_to_split.is_bg,
                    color_str: range_to_split.color_str,
                };

                if left_range.start <= left_range.end {
                    self.color_ranges.push(left_range);
                }
                if right_range.start <= right_range.end {
                    self.color_ranges.push(right_range);
                }
            }
        }

        self.color_ranges.push(ColorRange {
            start: cursor_x,
            end: cursor_x,
            is_bg: false,
            color_str: color::Black.fg_str(),
        });
        self.color_ranges.push(ColorRange {
            start: cursor_x,
            end: cursor_x,
            is_bg: true,
            color_str: color::White.bg_str(),
        });
    }

    pub fn render(&mut self) -> std::io::Result<()> {
        write!(self.writer, "{}", color::Reset.fg_str())?;
        write!(self.writer, "{}", color::Reset.bg_str())?;

        self.line
            .chars()
            .enumerate()
            .skip(self.display_range.0)
            .take(self.display_range.1 - self.display_range.0)
            .try_for_each(|(idx, ch)| self.render_line_char(idx, ch))?;

        if self.cursor_at_end {
            write!(self.writer, "{}", EMPTY_CURSOR)?;
        }

        write!(self.writer, "{}", color::Reset.fg_str())?;
        write!(self.writer, "{}", color::Reset.bg_str())?;

        Ok(())
    }
}
