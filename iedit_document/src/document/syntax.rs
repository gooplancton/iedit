use std::{fs, path::Path};

use regex_lite::Regex;

use crate::Document;

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
        color: String,
        pattern: Regex,
    },
    Multiline {
        color: String,
        start_pattern: Regex,
        end_pattern: Regex,
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
    pub fn infer_from_extension(ext: &str) -> Option<Self> {
        match ext {
            "py" => Some(Self::builtin_python()),
            "js" | "jsx" | "mjs" => Some(Self::builtin_javascript()),
            "rs" => Some(Self::builtin_rust()),
            "sh" | "bash" => Some(Self::builtin_bash()),
            "c" | "h" => Some(Self::builtin_c()),
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some(Self::builtin_cpp()),
            _ => None,
        }
    }

    pub fn from_file(path: impl AsRef<Path>) -> Option<Self> {
        let content = fs::read_to_string(path.as_ref()).ok()?;
        let mut lines = content.lines();

        let name = lines.next()?.trim();
        let name = Box::leak(name.to_string().into_boxed_str());

        let mut rules = Vec::new();

        for line in lines {
            let line = line.trim();
            if line.is_empty() || !line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            let color_hex = parts[0];
            let color = parse_color_hex(color_hex, false);
            if color.is_none() {
                continue;
            }

            let color = color.unwrap();

            if parts.len() == 2 {
                // Inline rule
                let pattern_str = parts[1];
                let pattern = Regex::new(pattern_str);
                if pattern.is_err() {
                    continue;
                }

                rules.push(SyntaxRule::Inline {
                    color,
                    pattern: pattern.unwrap(),
                });
            } else if parts.len() == 3 {
                // Multiline rule
                let start_pattern_str = parts[1];
                let end_pattern_str = parts[2];

                let start_pattern = Regex::new(start_pattern_str);
                let end_pattern = Regex::new(end_pattern_str);
                if start_pattern.is_err() || end_pattern.is_err() {
                    continue;
                }

                rules.push(SyntaxRule::Multiline {
                    color,
                    start_pattern: start_pattern.unwrap(),
                    end_pattern: end_pattern.unwrap(),
                });
            }
        }

        if rules.is_empty() {
            return None;
        }

        Some(DocumentSyntax { name, rules })
    }
}

pub fn parse_color_hex(color_hex: &str, is_bg: bool) -> Option<String> {
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

        self.lines
            .iter_mut()
            .for_each(|line| line.needs_render = true);

        let syntax = self.syntax.as_ref().unwrap();
        let mut cur_x = 0;
        let mut cur_y = 0;
        let mut blocks: Vec<SyntaxBlock> = vec![];
        let mut current_block: Option<SyntaxBlock> = None;

        'outer: while let Some(line) = self.lines.get(cur_y) {
            let mut block_closed = false;

            if let Some(current_block) = &mut current_block {
                // try to find the end of the current block

                if let Some(SyntaxRule::Multiline { end_pattern, .. }) =
                    syntax.rules.get(current_block.rule_idx)
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
                    if let SyntaxRule::Multiline { start_pattern, .. } = rule
                        && let Some(start_match) = start_pattern.find_at(line.as_ref(), cur_x)
                    {
                        let block_start_x = line
                            .byte_to_char_idx(start_match.start())
                            .unwrap_or(line.len());

                        current_block = Some(SyntaxBlock {
                            start_pos: (block_start_x, cur_y),
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

        self.syntax_blocks = blocks;
    }

    pub fn should_recompute_syntax_blocks(&mut self, affected_line_range: (usize, usize)) -> bool {
        // Naive check for now
        let mut multiline_patterns = self
            .syntax
            .as_ref()
            .map(|syntax| syntax.rules.as_slice())
            .unwrap_or_default()
            .iter()
            .filter_map(|rule| {
                if let SyntaxRule::Multiline {
                    start_pattern,
                    end_pattern,
                    ..
                } = rule
                {
                    Some([start_pattern, end_pattern])
                } else {
                    None
                }
            });

        for y in affected_line_range.0..=affected_line_range.1 {
            let line = self.lines.get(y);
            if line.is_none() {
                break;
            }

            let line = line.unwrap().as_ref();
            if self.syntax_blocks.iter().any(|block| block.intersects_y(y))
                || multiline_patterns.any(|[start_pattern, end_pattern]| {
                    start_pattern.is_match(line) || end_pattern.is_match(line)
                })
            {
                return true;
            }
        }

        false
    }
}
