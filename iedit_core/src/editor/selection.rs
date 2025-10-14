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

    pub fn highlight_chars(&self, chars: &[char]) -> String {
        let mut result = String::new();
        result.reserve(chars.len());

        match self {
            SelectionHighlight::None => {
                chars.iter().for_each(|ch| result.push(*ch));
            }
            SelectionHighlight::WholeLine => {
                let mut line_str = chars.iter().collect::<String>();

                result.push_str(HIGHLIGHT_START);
                chars.iter().for_each(|ch| result.push(*ch));
                result.push_str(HIGHLIGHT_END);
            }
            SelectionHighlight::Before(highlight_x) => {
                let x = min(*highlight_x, chars.len());
                let (highlighted, unhighlighted) = chars.split_at(x);

                result.push_str(HIGHLIGHT_START);
                highlighted.iter().for_each(|ch| result.push(*ch));
                result.push_str(HIGHLIGHT_END);
                unhighlighted.iter().for_each(|ch| result.push(*ch));
            }
            SelectionHighlight::After(highlight_x) => {
                let x = min(*highlight_x, chars.len());
                let (unhighlighted, highlighted) = chars.split_at(x);

                result.push_str(HIGHLIGHT_END);
                unhighlighted.iter().for_each(|ch| result.push(*ch));
                result.push_str(HIGHLIGHT_START);
                highlighted.iter().for_each(|ch| result.push(*ch));
            }
            SelectionHighlight::Range(highlight_start_x, highlight_end_x) => {
                let x1 = min(*highlight_start_x, chars.len());
                let (unhighlighted1, rest) = chars.split_at(x1);

                let x2 = min(
                    highlight_end_x.saturating_sub(unhighlighted1.len()),
                    rest.len(),
                );
                let (highlighted, unhighlighted2) = rest.split_at(x2);

                unhighlighted1.iter().for_each(|ch| result.push(*ch));
                result.push_str(HIGHLIGHT_START);
                highlighted.iter().for_each(|ch| result.push(*ch));
                result.push_str(HIGHLIGHT_END);
                unhighlighted2.iter().for_each(|ch| result.push(*ch));
            }
        };

        result
    }
}
