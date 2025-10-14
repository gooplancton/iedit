use std::{array::IntoIter, cmp::min, io::Write, ops::Deref};

use crate::terminal::{HIGHLIGHT_END, HIGHLIGHT_START};

pub type SelectionRage = ((usize, usize), (usize, usize));
pub enum SelectionHighlight {
    None,
    WholeLine,
    Before(usize),
    After(usize),
    Range(usize, usize),
}

impl SelectionHighlight {
    pub fn from_line_idx_and_selection_range(
        line_idx: usize,
        selection_range: &SelectionRage,
    ) -> Self {
        let ((start_x, start_y), (end_x, end_y)) = *selection_range;
        if start_y == end_y && start_y == line_idx {
            SelectionHighlight::Range(start_x, end_x)
        } else if start_y == line_idx {
            SelectionHighlight::After(start_x)
        } else if end_y == line_idx {
            SelectionHighlight::Before(end_x)
        } else if start_y < line_idx && end_y > line_idx {
            SelectionHighlight::WholeLine
        } else {
            SelectionHighlight::None
        }
    }
}

pub struct HighlightedStringChunks<'a>([Option<&'a str>; 5]);

impl<'a> HighlightedStringChunks<'a> {
    pub fn from(string: &'a str, selection_highlight: &'a SelectionHighlight) -> Self {
        let chunks = match selection_highlight {
            SelectionHighlight::None => [Some(string), None, None, None, None],
            SelectionHighlight::WholeLine => [
                Some(HIGHLIGHT_START),
                Some(string),
                Some(HIGHLIGHT_END),
                None,
                None,
            ],
            SelectionHighlight::Before(highlight_x) => {
                let x = min(*highlight_x, string.len());
                let (highlighted, unhighlighted) = string.split_at(x);
                [
                    Some(HIGHLIGHT_START),
                    Some(highlighted),
                    Some(HIGHLIGHT_END),
                    Some(unhighlighted),
                    None,
                ]
            }
            SelectionHighlight::After(highlight_x) => {
                let x = min(*highlight_x, string.len());
                let (unhighlighted, highlighted) = string.split_at(x);
                [
                    Some(unhighlighted),
                    Some(HIGHLIGHT_START),
                    Some(highlighted),
                    Some(HIGHLIGHT_END),
                    None,
                ]
            }
            SelectionHighlight::Range(highlight_start_x, highlight_end_x) => {
                let x1 = min(*highlight_start_x, string.len());
                let (unhighlighted1, rest) = string.split_at(x1);

                let x2 = min(
                    highlight_end_x.saturating_sub(unhighlighted1.len()),
                    rest.len(),
                );
                let (highlighted, unhighlighted2) = rest.split_at(x2);
                [
                    Some(unhighlighted1),
                    Some(HIGHLIGHT_START),
                    Some(highlighted),
                    Some(HIGHLIGHT_END),
                    Some(unhighlighted2),
                ]
            }
        };

        Self(chunks)
    }

    pub fn write_to(&self, mut target: impl Write) -> std::io::Result<()> {
        for chunk in self.0 {
            if chunk.is_none() {
                return Ok(());
            }

            target.write_all(chunk.unwrap().as_bytes())?
        }

        Ok(())
    }
}
