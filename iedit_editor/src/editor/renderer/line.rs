use std::io::Write;

use crate::{
    editor::highlight::{SelectionHighlight, SyntaxHighlight},
    terminal::EMPTY_CURSOR,
};
use iedit_document::CharacterIndexable;
use termion::color;

pub struct ColorRange<'renderer> {
    start: usize,
    /// (inclusive)
    end: usize,
    is_bg: bool,
    color_str: &'renderer str,
}

pub struct LineRenderer<'line, 'writer, Writer: Write> {
    pub line: &'line str,
    pub display_range: (usize, usize),
    pub writer: &'writer mut Writer,
    pub color_ranges: Vec<ColorRange<'writer>>,
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
                if range.end < range.start {
                    return false
                }

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
            .filter(|range| range.end == idx && range.end >= range.start)
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
                    end: end.saturating_sub(1),
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
                    end: end.saturating_sub(1),
                    is_bg: true,
                    color_str: color::LightBlue.bg_str(),
                });
            }
        }
    }

    pub fn add_syntax_highlight(&mut self, syntax_highlight: &'writer SyntaxHighlight) {
        for rule in &syntax_highlight.rules {
            for rx_match in rule.pattern.find_iter(self.line) {
                let start_char = self
                    .line
                    .byte_to_char_idx(rx_match.start())
                    .unwrap_or_default();
                let end_char = self
                    .line
                    .byte_to_char_idx(rx_match.end() - 1)
                    .unwrap_or(self.line.n_chars());
                self.color_ranges.push(ColorRange {
                    start: start_char,
                    end: end_char,
                    is_bg: rule.is_bg,
                    color_str: &rule.color,
                });
            }
        }
    }

    pub fn add_cursor(&mut self, cursor_x: usize) {
        if cursor_x >= self.line.len() {
            self.cursor_at_end = true;
            return;
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
            .take(self.display_range.1.saturating_sub(self.display_range.0))
            .try_for_each(|(idx, ch)| self.render_line_char(idx, ch))?;

        if self.cursor_at_end {
            write!(self.writer, "{}", EMPTY_CURSOR)?;
        }

        write!(self.writer, "{}", color::Reset.fg_str())?;
        write!(self.writer, "{}", color::Reset.bg_str())?;

        Ok(())
    }
}
