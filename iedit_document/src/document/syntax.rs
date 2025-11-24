use regex_lite::Regex;

use crate::{Document, EditOperation};

pub struct DocumentSyntax {
    pub name: &'static str,
    pub rules: Vec<SyntaxRule>,
}

#[derive(Debug)]
pub struct SyntaxBlock {
    pub start_pos: (usize, usize),
    pub end_pos: Option<(usize, usize)>,
    pub end_symbol_len: usize,
    pub rule_idx: usize,
}

impl SyntaxBlock {
    #[inline]
    pub fn contains_pos(&self, pos: (usize, usize)) -> bool {
        (pos.1 > self.start_pos.1 || (pos.1 == self.start_pos.1 && pos.0 >= self.start_pos.0))
            && self.end_pos.is_none_or(|end_pos| {
                pos.1 < end_pos.1 || (pos.1 == end_pos.1 && pos.0 <= end_pos.0)
            })
    }

    #[inline]
    pub fn intersects_y(&self, y: usize) -> bool {
        y >= self.start_pos.1 && self.end_pos.is_none_or(|pos| y <= pos.1)
    }
}

pub enum SyntaxRule {
    Inline {
        pattern: Regex,
        color: String,
    },
    Multiline {
        start_pattern: Regex,
        end_pattern: Regex,
        color: String,
    },
}

impl SyntaxRule {
    #[inline]
    pub fn get_color(&self) -> &str {
        match self {
            SyntaxRule::Inline { pattern: _, color } => color,
            SyntaxRule::Multiline {
                start_pattern: _,
                end_pattern: _,
                color,
            } => color,
        }
    }
}

impl DocumentSyntax {
    pub fn infer_from_extension(ext: &std::ffi::OsStr) -> Option<Self> {
        // if let Some(base_path) = base_path {
        //     let custom_sytnax_highlight = Self::load_from_file(
        //         base_path
        //             .join(format!("{}.syntax.conf", ext.to_str()?))
        //             .as_path(),
        //     );

        //     if let Some(syntax_highlight) = custom_sytnax_highlight {
        //         return Some(syntax_highlight);
        //     }
        // }

        match ext.to_str()? {
            "py" => Some(Self::builitn_python()),
            _ => None,
        }
    }

    // pub fn load_from_file(path: impl AsRef<Path>) -> Option<Self> {
    //     todo!()
    // }

    pub fn builitn_python() -> Self {
        Self {
            name: "Python",
            rules: vec![
                // Comments
                SyntaxRule::Inline {
                    pattern: Regex::new(r"^#.*$").unwrap(),
                    color: parse_color_hex("#6A9955", false).unwrap(),
                },
                SyntaxRule::Multiline {
                    start_pattern: Regex::new(r#"""""#).unwrap(),
                    end_pattern: Regex::new(r#"""""#).unwrap(),
                    color:  parse_color_hex("#6A9955", false).unwrap(),
                },
                SyntaxRule::Multiline {
                    start_pattern: Regex::new(r#"'''"#).unwrap(),
                    end_pattern: Regex::new(r#"'''"#).unwrap(),
                    color:  parse_color_hex("#6A9955", false).unwrap(),
                },
                // Strings
                SyntaxRule::Inline {
                    pattern: Regex::new(r#"^"(?:[^"\\]|\\.)*""#).unwrap(),
                    color: parse_color_hex("#6A9955", false).unwrap()
                },
                SyntaxRule::Inline {
                    pattern: Regex::new(r#"^'(?:[^'\\]|\\.)*'"#).unwrap(),
                    color: parse_color_hex("#6A9955", false).unwrap()
                },
                // Keywords
                SyntaxRule::Inline {
                    pattern: Regex::new(r"^\b(def|class|if|else|elif|for|while|return|import|from|as|try|except|finally|with|lambda)\b").unwrap(),
                    color: parse_color_hex("#569CD6", false).unwrap(),
                },
                // Built-in functions and types
                SyntaxRule::Inline {
                    pattern: Regex::new(r"^\b(print|len|str|int|float|list|dict|set|range|type|isinstance)\b").unwrap(),
                    color: parse_color_hex("#85855d", false).unwrap(),
                },
                // Numbers
                SyntaxRule::Inline {
                    pattern: Regex::new(r"^\b\d+\b").unwrap(),
                    color: parse_color_hex("#B5CEA8", false).unwrap(),
                },
                // Operators
                SyntaxRule::Inline {
                    pattern: Regex::new(r"^[=+\-*/<>!&|]+").unwrap(),
                    color: parse_color_hex("#D4D4D4", false).unwrap()
                },
            ],
        }
    }
}

// rules: &[
//     // Comments
//

fn parse_color_hex(color_hex: &str, is_bg: bool) -> Option<String> {
    if color_hex.starts_with('#')
        && color_hex.len() == 7
        && let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&color_hex[1..3], 16),
            u8::from_str_radix(&color_hex[3..5], 16),
            u8::from_str_radix(&color_hex[5..7], 16),
        )
    {
        if is_bg {
            Some(termion::color::Rgb(r, g, b).bg_string())
        } else {
            Some(termion::color::Rgb(r, g, b).fg_string())
        }
    } else {
        None
    }
}

impl Document {
    pub fn recompute_syntax_blocks(&mut self) {
        if self.syntax.is_none() {
            return;
        }

        let syntax = self.syntax.as_ref().unwrap();
        let mut cur_x = 0;
        let mut cur_y = 0;
        let mut blocks: Vec<SyntaxBlock> = vec![];
        let mut current_block: Option<SyntaxBlock> = None;

        'outer: while let Some(line) = self.lines.get(cur_y) {
            let mut block_closed = false;

            if let Some(current_block) = &mut current_block {
                // try to find the end of the current block

                if let Some(SyntaxRule::Multiline {
                    start_pattern: _,
                    end_pattern,
                    color: _,
                }) = syntax.rules.get(current_block.rule_idx)
                {
                    if let Some(end_match) = end_pattern.find_at(line.as_ref(), cur_x) {
                        let block_end_x_start = line
                            .byte_to_char_idx(end_match.start())
                            .unwrap_or(line.len());
                        let block_end_x_end =
                            line.byte_to_char_idx(end_match.end()).unwrap_or(line.len());

                        current_block.end_symbol_len = block_end_x_end - block_end_x_start;
                        current_block.end_pos = Some((block_end_x_end, cur_y));
                        cur_x = block_end_x_end + 1;
                        block_closed = true;
                    } else {
                        cur_x = 0;
                        cur_y += 1;
                    }
                }
            } else {
                // try to find a block start

                for (rule_idx, rule) in syntax.rules.iter().enumerate() {
                    if let SyntaxRule::Multiline {
                        start_pattern,
                        end_pattern: _,
                        color: _,
                    } = rule
                        && let Some(start_match) = start_pattern.find_at(line.as_ref(), cur_x)
                    {
                        let block_start_x = line
                            .byte_to_char_idx(start_match.start())
                            .unwrap_or(line.len());

                        current_block = Some(SyntaxBlock {
                            start_pos: ((block_start_x, cur_y)),
                            end_pos: None,
                            end_symbol_len: 0,
                            rule_idx,
                        });
                        cur_x = block_start_x + 1;

                        continue 'outer;
                    }
                }

                cur_x = 0;
                cur_y += 1;
            }

            if block_closed {
                blocks.push(current_block.take().unwrap());
            }
        }

        if current_block.is_some() {
            // unclosed block spans till document end
            blocks.push(current_block.take().unwrap());
        }

        eprintln!("debug: syntax blocks: {:?}", &blocks);

        self.syntax_blocks = blocks;
    }

    pub fn should_recompute_syntax_blocks(&mut self, op: &EditOperation) -> bool {
        // Naive check for now
        match *op {
            EditOperation::LineRemoval { idx: y }
            | EditOperation::Deletion { pos: (_, y) }
            | EditOperation::Insertion {
                pos: (_, y),
                text: _,
            } => self.syntax_blocks.iter().any(|block| block.intersects_y(y)),
            EditOperation::Replacement {
                pos_from: (_, y_from),
                pos_to: (_, y_to),
                text: _,
            } => self.syntax_blocks.iter().any(|block| {
                block.intersects_y(y_from)
                    || block.intersects_y(y_to)
                    || (y_from < block.start_pos.1 && block.end_pos.is_none_or(|pos| y_to > pos.1))
            }),
        }
    }
}
