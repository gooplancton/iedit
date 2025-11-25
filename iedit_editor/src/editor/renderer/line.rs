use std::io::Write;

use crate::{
    editor::highlight::RangeHighlight,
    terminal::{self, EMPTY_CURSOR},
};
use iedit_document::{DocumentLine, DocumentSyntax, SyntaxBlock, SyntaxRule};
use termion::color::{self};

pub struct ColorRange<'renderer> {
    start: usize,
    /// (inclusive)
    end: usize,
    is_bg: bool,
    color_str: &'renderer str,
}

pub struct LineRenderer<'line, 'writer, Writer: Write> {
    pub line: &'line DocumentLine,
    pub line_idx: usize,
    pub char_offset: usize,
    pub trailing_whitespace_offset: usize,
    pub visual_offset: usize,
    pub ui_width: usize,
    pub writer: &'writer mut Writer,
    pub color_ranges: Vec<ColorRange<'writer>>,
    pub tab_size: usize,
    pub cursor_at_end: bool,
}

impl<'line, 'writer, Writer: Write> LineRenderer<'line, 'writer, Writer> {
    pub fn new(
        line: &'line DocumentLine,
        line_idx: usize,
        visual_offset: usize,
        ui_width: usize,
        writer: &'writer mut Writer,
        tab_size: usize,
    ) -> Self {
        let char_offset = line.visual_to_char_idx(visual_offset, tab_size);

        Self {
            line,
            line_idx,
            char_offset,
            visual_offset,
            ui_width,
            color_ranges: vec![],
            trailing_whitespace_offset: line.len(),
            writer,
            tab_size,
            cursor_at_end: false,
        }
    }

    fn render_line_char(
        &mut self,
        ch: char,
        char_idx: usize,
        visual_idx: usize,
    ) -> std::io::Result<()> {
        self.color_ranges
            .iter()
            .filter(|range| {
                if range.end < range.start {
                    return false;
                }

                if char_idx == self.char_offset {
                    range.start <= char_idx
                } else {
                    range.start == char_idx
                }
            })
            .try_for_each(|range| write!(self.writer, "{}", range.color_str))?;

        if ch == '\t' {
            let n_spaces = self.tab_size - (visual_idx % self.tab_size);
            let tab_string = if char_idx < self.trailing_whitespace_offset {
                " ".repeat(n_spaces)
            } else {
                "─".repeat(n_spaces)
            };
            write!(self.writer, "{}", &tab_string)?;
        } else if char_idx < self.trailing_whitespace_offset {
            write!(self.writer, "{}", ch)?;
        } else {
            write!(self.writer, "•")?;
        }

        self.color_ranges
            .iter()
            .filter(|range| range.end == char_idx && range.end >= range.start)
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

    pub fn add_trailing_whitespace(&mut self) {
        let trailing_whitespace_idx = self
            .line
            .as_ref()
            .rfind(|ch: char| !ch.is_whitespace())
            .map(|idx| idx + 1)
            .unwrap_or_default();
        self.trailing_whitespace_offset = self
            .line
            .byte_to_char_idx(trailing_whitespace_idx)
            .unwrap_or(self.line.len());
    }

    pub fn add_range_highlight(
        &mut self,
        range_highlight: RangeHighlight,
        is_bg: bool,
        color_str: &'writer str,
    ) {
        match range_highlight {
            RangeHighlight::None => {}
            RangeHighlight::After(start) => {
                self.color_ranges.push(ColorRange {
                    start,
                    end: self.line.len(),
                    is_bg,
                    color_str,
                });
            }
            RangeHighlight::Before(end) => {
                self.color_ranges.push(ColorRange {
                    start: 0,
                    end: end.saturating_sub(1),
                    is_bg,
                    color_str,
                });
            }
            RangeHighlight::WholeLine => {
                self.color_ranges.push(ColorRange {
                    start: 0,
                    end: self.line.len(),
                    is_bg,
                    color_str,
                });
            }
            RangeHighlight::Range(start, end) => {
                self.color_ranges.push(ColorRange {
                    start,
                    end: end.saturating_sub(1),
                    is_bg,
                    color_str,
                });
            }
        }
    }

    pub fn add_syntax_highlight(
        &mut self,
        syntax: &'writer DocumentSyntax,
        blocks: &'writer [SyntaxBlock],
    ) {
        let mut offset = 0;
        let len = self.line.len();
        let mut iter = 0;

        'outer: while offset < len && iter < len {
            iter += 1;
            for block in blocks.iter() {
                let color_str = &syntax.rules[block.rule_idx].get_color();
                if (offset, self.line_idx) == block.start_pos {
                    // start of block
                    if let Some(end_pos) = block.end_pos
                        && end_pos.1 == self.line_idx
                    {
                        // inline block, skip to block end
                        self.color_ranges.push(ColorRange {
                            start: offset,
                            end: end_pos.0.saturating_sub(1),
                            is_bg: false,
                            color_str,
                        });

                        offset = end_pos.0;
                        continue 'outer;
                    } else {
                        // start of block, end is on another line
                        self.color_ranges.push(ColorRange {
                            start: offset,
                            end: self.line.len(),
                            is_bg: false,
                            color_str,
                        });
                        break 'outer;
                    }
                } else if block.end_pos.is_some_and(|end_pos| {
                    (end_pos.0 - block.end_symbol_len, end_pos.1) == (offset, self.line_idx)
                }) {
                    // end of block, start_pos.0 is for sure on a previous line
                    self.color_ranges.push(ColorRange {
                        start: 0,
                        end: offset + block.end_symbol_len.saturating_sub(1),
                        is_bg: false,
                        color_str,
                    });
                    offset += block.end_symbol_len + 1;
                    continue 'outer;
                } else if block.contains_pos((offset, self.line_idx)) {
                    // inside a block
                    let end = if let Some(pos) = block.end_pos
                        && pos.1 == self.line_idx
                    {
                        // block ends in current line
                        pos.0
                    } else {
                        // block extends to next line
                        self.line.len()
                    };
                    self.color_ranges.push(ColorRange {
                        start: 0,
                        end: end.saturating_sub(1),
                        is_bg: false,
                        color_str,
                    });
                    offset = end + 1;
                    continue 'outer;
                }
            }

            let offset_byte_idx = self.line.byte_to_char_idx(offset);
            if offset_byte_idx.is_none() {
                return;
            }

            let offset_byte_idx = offset_byte_idx.unwrap();
            for rule in syntax.rules.iter() {
                if let SyntaxRule::Inline { pattern, color } = rule {
                    if let Some(rx_match) =
                        pattern.find_anchored_at(self.line.as_ref(), offset_byte_idx)
                    {
                        let start_char = self
                            .line
                            .byte_to_char_idx(rx_match.start().saturating_sub(1))
                            .unwrap_or_default();
                        let end_char = self
                            .line
                            .byte_to_char_idx(rx_match.end().saturating_sub(1))
                            .unwrap_or(self.line.len());

                        self.color_ranges.push(ColorRange {
                            start: start_char,
                            end: end_char,
                            is_bg: false,
                            color_str: &color,
                        });

                        offset = end_char + 1;
                        continue 'outer;
                    }
                }
            }

            offset += 1;
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
        self.color_ranges
            .iter()
            .filter(|range| range.start <= range.end && range.start <= self.char_offset)
            .try_for_each(|range| write!(self.writer, "{}", range.color_str))?;

        if self.visual_offset % self.tab_size != 0 && self.line.at(self.char_offset) == Some('\t') {
            let hidden_length = self.visual_offset % self.tab_size;
            let partial_tab_length = self.tab_size - hidden_length;
            write!(self.writer, "{}", " ".repeat(partial_tab_length))?;
        }

        for (char_idx, ch) in self.line.iter().enumerate() {
            let visual_idx = self.line.char_to_visual_idx(char_idx, self.tab_size);

            if visual_idx >= self.ui_width + self.visual_offset {
                break;
            }

            if char_idx >= self.trailing_whitespace_offset {
                write!(self.writer, "{}", termion::color::LightBlack.fg_str())?;
            }

            if visual_idx >= self.visual_offset {
                self.render_line_char(ch, char_idx, visual_idx)?;
            }
        }

        if self.cursor_at_end {
            write!(self.writer, "{}", EMPTY_CURSOR)?;
        }

        write!(self.writer, "{}", color::Reset.fg_str())?;
        write!(self.writer, "{}", color::Reset.bg_str())?;
        write!(self.writer, "{}", terminal::CLEAR_TO_END_OF_LINE)?;

        Ok(())
    }
}
