
use std::{cmp::min, io::Write, ops::Range};

use crate::{
    line::{CharacterEditable, DocumentLine},
    terminal::{HIGHLIGHT_END, HIGHLIGHT_START},
    editor::highlight::SelectionHighlight
};

pub struct LineRenderer<'line> {
    pub line: &'line String,
    pub display_range: Range<usize>,
    pub selection_highlight: SelectionHighlight,
}

impl<'line> LineRenderer<'line> {
    pub fn new(line: &'line String) -> Self {
        Self {
            line,
            display_range: (0..line.n_chars()),
            selection_highlight: SelectionHighlight::None,
        }
    }

    pub fn with_display_range(mut self, display_range: Range<usize>) -> Self {
        self.display_range = display_range;

        self
    }

    pub fn with_selection_highlight(mut self, selection_highlight: SelectionHighlight) -> Self {
        self.selection_highlight = selection_highlight;

        self
    }

    pub fn render_to(&mut self, writer: &mut impl Write) -> std::io::Result<()> {
        let content = self.line.get_chars(self.display_range.clone());

        match self.selection_highlight {
            SelectionHighlight::None => {
                content
                    .iter_chars()
                    .try_for_each(|ch| write!(writer, "{}", ch))?;
            }
            SelectionHighlight::WholeLine => {
                writer.write_all(HIGHLIGHT_START.as_bytes())?;
                content
                    .iter_chars()
                    .try_for_each(|ch| write!(writer, "{}", ch))?;
                writer.write_all(HIGHLIGHT_END.as_bytes())?;
            }
            SelectionHighlight::Before(highlight_x) => {
                let x = min(highlight_x, content.n_chars());
                let (highlighted, unhighlighted) = content.split_chars_at(x);

                writer.write_all(HIGHLIGHT_START.as_bytes())?;
                highlighted
                    .iter_chars()
                    .try_for_each(|ch| write!(writer, "{}", ch))?;
                writer.write_all(HIGHLIGHT_END.as_bytes())?;
                unhighlighted
                    .iter_chars()
                    .try_for_each(|ch| write!(writer, "{}", ch))?;
            }
            SelectionHighlight::After(highlight_x) => {
                let x = min(highlight_x, content.n_chars());
                let (unhighlighted, highlighted) = content.split_chars_at(x);

                writer.write_all(HIGHLIGHT_END.as_bytes())?;
                unhighlighted
                    .iter_chars()
                    .try_for_each(|ch| write!(writer, "{}", ch))?;
                writer.write_all(HIGHLIGHT_START.as_bytes())?;
                highlighted
                    .iter_chars()
                    .try_for_each(|ch| write!(writer, "{}", ch))?;
            }
            SelectionHighlight::Range(highlight_start_x, highlight_end_x) => {
                let x1 = min(highlight_start_x, content.n_chars());
                let (unhighlighted1, rest) = content.split_chars_at(x1);

                let x2 = min(
                    highlight_end_x.saturating_sub(unhighlighted1.n_chars()),
                    rest.n_chars(),
                );
                let (highlighted, unhighlighted2) = rest.split_chars_at(x2);

                unhighlighted1
                    .iter_chars()
                    .try_for_each(|ch| write!(writer, "{}", ch))?;

                writer.write_all(HIGHLIGHT_START.as_bytes())?;

                highlighted
                    .iter_chars()
                    .try_for_each(|ch| write!(writer, "{}", ch))?;

                writer.write_all(HIGHLIGHT_END.as_bytes())?;

                unhighlighted2
                    .iter_chars()
                    .try_for_each(|ch| write!(writer, "{}", ch))?;
            }
        };

        Ok(())
    }
}
