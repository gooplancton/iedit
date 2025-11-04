use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

use termion::color;

/// selection types kept as before
pub type SelectionRage = ((usize, usize), (usize, usize));
pub enum SelectionHighlight {
    None,
    WholeLine,
    Before(usize),
    After(usize),
    Range(usize, usize),
}

impl SelectionHighlight {
    pub fn new(line_idx: usize, selection_range: &SelectionRage) -> Self {
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

/// A small, renderer-friendly span produced by syntect highlighting:
/// start/end are character indices (inclusive).
pub struct HighlightSpan {
    pub start: usize,
    pub end: usize,
    /// foreground escape sequence (termion)
    pub fg_escape: String,
    /// optional background escape sequence (termion)
    pub bg_escape: Option<String>,
}

pub struct SyntectHighlighter {
    syntax_set: SyntaxSet,
    theme: Theme,
}

impl SyntectHighlighter {
    pub fn new<'theme_name>(syntax_set: SyntaxSet) -> Self {
        let ts = ThemeSet::load_defaults();
        // fallback if requested theme not found
        let mut theme = ts
            .themes
            .get("base16-ocean.dark")
            .expect("default theme missing")
            .clone();

        theme.settings.background = Some(Color::BLACK);

        Self { syntax_set, theme }
    }

    /// Highlight a single logical line. `file_path` is optional and used to pick syntax by extension.
    /// Returns spans with character indices (not byte indices).
    pub fn highlight_line(&self, syntax: &SyntaxReference, line: &str) -> Vec<HighlightSpan> {
        // pick syntax by extension or fallback to plain text

        let mut h = HighlightLines::new(syntax, &self.theme);

        // syntect works with bytes/slices; highlight_line returns Vec<(Style, &str)> where the &str are slices
        let regions = h.highlight_line(line, &self.syntax_set);
        if regions.is_err() {
            return Vec::new();
        }

        let mut spans = Vec::new();
        let mut char_cursor = 0usize;

        for (style, text) in regions.unwrap() {
            let n_chars = text.chars().count();
            if n_chars == 0 {
                continue;
            }

            let fg = style.foreground;
            let bg = style.background;

            let fg_escape = format!("{}", color::Fg(color::Rgb(fg.r, fg.g, fg.b)));
            let bg_escape = if bg.a != 0 {
                Some(format!("{}", color::Bg(color::Rgb(bg.r, bg.g, bg.b))))
            } else {
                None
            };

            let span = HighlightSpan {
                start: char_cursor,
                end: char_cursor + n_chars.saturating_sub(1),
                fg_escape,
                bg_escape,
            };
            spans.push(span);

            char_cursor += n_chars;
        }

        spans
    }
}
