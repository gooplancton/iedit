use regex_lite::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

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

#[derive(Debug, Clone)]
pub struct SyntaxRule {
    pub pattern: Regex,
    pub color: String,
    pub is_bg: bool,
}

#[derive(Default, Debug)]
pub struct SyntaxHighlight {
    pub rules: Vec<SyntaxRule>,
}

impl SyntaxHighlight {
    fn builtin_rust() -> Self {
        let mut rules = Vec::new();

        // Keywords - steel blue
        rules.push(SyntaxRule {
            pattern: Regex::new(r"\b(fn|let|mut|pub|use|struct|enum|impl|trait|where|match|if|else|for|loop|while|return|mod|type|const|static|extern|async|await|move|ref|unsafe|in|continue|break|dyn|box|super|self|Self|crate|as)\b").unwrap(),
            color: termion::color::Rgb(143, 161, 179).fg_string(),
            is_bg: false,
        });

        // Types - soft cyan
        rules.push(SyntaxRule {
            pattern: Regex::new(r"\b(i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize|f32|f64|bool|char|str|String|Vec|Option|Result|Box)\b").unwrap(),
            color: termion::color::Rgb(150, 181, 180).fg_string(),
            is_bg: false,
        });

        // Comments - muted gray
        rules.push(SyntaxRule {
            pattern: Regex::new(r"//.*$|/\*(?:[^*]|\*[^/])*\*/").unwrap(),
            color: termion::color::Rgb(101, 115, 126).fg_string(),
            is_bg: false,
        });

        // Strings - soft orange
        rules.push(SyntaxRule {
            pattern: Regex::new(r#""([^"\\]|\\.)*"|'([^'\\]|\\.)*'"#).unwrap(),
            color: termion::color::Rgb(208, 135, 112).fg_string(),
            is_bg: false,
        });

        // Numbers - soft purple
        rules.push(SyntaxRule {
            pattern: Regex::new(r"\b\d[\d_]*(\.\d[\d_]*)?([eE][+-]?\d[\d_]*)?\b").unwrap(),
            color: termion::color::Rgb(180, 142, 173).fg_string(),
            is_bg: false,
        });

        Self { rules }
    }

    fn builtin_python() -> Self {
        let mut rules = Vec::new();

        // Keywords - steel blue
        rules.push(SyntaxRule {
            pattern: Regex::new(r"\b(def|class|if|else|elif|for|while|return|import|from|as|try|except|finally|with|lambda|yield|raise|break|continue|pass|assert|del|global|nonlocal|in|is|not|and|or)\b").unwrap(),
            color: termion::color::Rgb(143, 161, 179).fg_string(),
            is_bg: false,
        });

        // Built-in functions and types - soft cyan
        rules.push(SyntaxRule {
            pattern: Regex::new(r"\b(print|len|str|int|float|list|dict|set|tuple|bool|range|enumerate|zip|map|filter|any|all|sum|min|max|sorted|reversed|iter|next|super|isinstance|hasattr|getattr|setattr)\b").unwrap(),
            color: termion::color::Rgb(150, 181, 180).fg_string(),
            is_bg: false,
        });

        // Comments - muted gray
        rules.push(SyntaxRule {
            pattern: Regex::new(r"#.*$").unwrap(),
            color: termion::color::Rgb(101, 115, 126).fg_string(),
            is_bg: false,
        });

        // Strings - soft orange
        rules.push(SyntaxRule {
            pattern: Regex::new(
                r#"("""[\s\S]*?""")|'''[\s\S]*?'''|"([^"\\]|\\.)*"|'([^'\\]|\\.)*'"#,
            )
            .unwrap(),
            color: termion::color::Rgb(208, 135, 112).fg_string(),
            is_bg: false,
        });

        Self { rules }
    }

    fn builtin_javascript() -> Self {
        let mut rules = Vec::new();

        // Keywords - steel blue
        rules.push(SyntaxRule {
            pattern: Regex::new(r"\b(const|let|var|function|class|if|else|for|while|do|switch|case|break|continue|return|try|catch|finally|throw|new|delete|typeof|instanceof|void|yield|async|await|of|in)\b").unwrap(),
            color: termion::color::Rgb(143, 161, 179).fg_string(),
            is_bg: false,
        });

        // Built-in objects and functions - soft cyan
        rules.push(SyntaxRule {
            pattern: Regex::new(r"\b(Array|Object|String|Number|Boolean|Function|RegExp|Date|Promise|Map|Set|Symbol|console|Math|JSON)\b").unwrap(),
            color: termion::color::Rgb(150, 181, 180).fg_string(),
            is_bg: false,
        });

        // Comments - muted gray
        rules.push(SyntaxRule {
            pattern: Regex::new(r"//.*$|/\*(?:[^*]|\*[^/])*\*/").unwrap(),
            color: termion::color::Rgb(101, 115, 126).fg_string(),
            is_bg: false,
        });

        // Strings - soft orange
        rules.push(SyntaxRule {
            pattern: Regex::new(r#""([^"\\]|\\.)*"|'([^'\\]|\\.)*'|`([^`\\]|\\.)*`"#).unwrap(),
            color: termion::color::Rgb(208, 135, 112).fg_string(),
            is_bg: false,
        });

        Self { rules }
    }

    pub fn infer_from_extension(ext: &std::ffi::OsStr, base_path: Option<PathBuf>) -> Option<Self> {
        if let Some(base_path) = base_path {
            let custom_sytnax_highlight = Self::load_from_nanorc(
                base_path
                    .join(format!("{}.nanorc", ext.to_str()?))
                    .as_path(),
            );

            if let Ok(syntax_highlight) = custom_sytnax_highlight {
                return Some(syntax_highlight);
            }
        }

        match ext.to_str()? {
            "rs" => Some(Self::builtin_rust()),
            "py" => Some(Self::builtin_python()),
            "js" | "jsx" | "ts" | "tsx" => Some(Self::builtin_javascript()),
            _ => None,
        }
    }

    pub fn load_from_nanorc(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut highlighter = Self::default();
        // current_color not used as stateful token in this version
        for raw_line in reader.lines() {
            let line = raw_line?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // split into directive, color-token, rest-of-line (pattern)
            let mut parts = line.splitn(3, char::is_whitespace);
            let directive = parts.next().unwrap_or("").to_lowercase();
            if directive != "color" && directive != "icolor" {
                continue;
            }
            let color_token = parts.next().unwrap_or("").trim();
            let mut pattern = parts.next().unwrap_or("").trim().to_string();
            if pattern.is_empty() {
                continue;
            }

            // strip surrounding quotes; some nanorc files use doubled quotes ""..."" — strip repeatedly
            while pattern.len() >= 2 && pattern.starts_with('"') && pattern.ends_with('"') {
                pattern = pattern[1..pattern.len() - 1].to_string();
            }
            // also handle single-quoted patterns
            while pattern.len() >= 2 && pattern.starts_with('\'') && pattern.ends_with('\'') {
                pattern = pattern[1..pattern.len() - 1].to_string();
            }

            // detect background flag when color token starts with ','
            let is_bg = color_token.starts_with(',');
            let color_name = if is_bg {
                &color_token[1..]
            } else {
                color_token
            };

            // build the rust regex string — honor icolor (case-insensitive) with (?i) prefix
            let mut re_pat = pattern;
            if directive == "icolor" {
                re_pat = format!("(?i){}", re_pat);
            }

            // convert a couple of nanorc specific escapes to Rust-friendly equivalents
            let re_pat = re_pat.replace(r"\<", r"\b").replace(r"\>", r"\b");

            // compile regex
            match Regex::new(&re_pat) {
                Ok(regex) => {
                    let color_escape = Self::parse_color(color_name, is_bg);
                    highlighter.rules.push(SyntaxRule {
                        pattern: regex,
                        color: color_escape,
                        is_bg,
                    });
                }
                Err(_) => {
                    // skip invalid patterns silently
                    continue;
                }
            }
        }

        Ok(highlighter)
    }

    fn parse_color(color: &str, as_bg: bool) -> String {
        // Using base16-ocean.dark inspired colors
        match color.to_lowercase().as_str() {
            "red" => {
                if as_bg {
                    termion::color::Rgb(191, 97, 106).bg_string()
                } else {
                    termion::color::Rgb(191, 97, 106).fg_string()
                }
            }
            "green" => {
                if as_bg {
                    termion::color::Rgb(163, 190, 140).bg_string()
                } else {
                    termion::color::Rgb(163, 190, 140).fg_string()
                }
            }
            "blue" => {
                if as_bg {
                    termion::color::Rgb(143, 161, 179).bg_string()
                } else {
                    termion::color::Rgb(143, 161, 179).fg_string()
                }
            }
            "yellow" => {
                if as_bg {
                    termion::color::Rgb(235, 203, 139).bg_string()
                } else {
                    termion::color::Rgb(235, 203, 139).fg_string()
                }
            }
            "magenta" => {
                if as_bg {
                    termion::color::Rgb(180, 142, 173).bg_string()
                } else {
                    termion::color::Rgb(180, 142, 173).fg_string()
                }
            }
            "cyan" => {
                if as_bg {
                    termion::color::Rgb(150, 181, 180).bg_string()
                } else {
                    termion::color::Rgb(150, 181, 180).fg_string()
                }
            }
            "orange" => {
                if as_bg {
                    termion::color::Rgb(208, 135, 112).bg_string()
                } else {
                    termion::color::Rgb(208, 135, 112).fg_string()
                }
            }
            // Support for RGB format: color #RRGGBB
            _ if color.starts_with('#') && color.len() == 7 => {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&color[1..3], 16),
                    u8::from_str_radix(&color[3..5], 16),
                    u8::from_str_radix(&color[5..7], 16),
                ) {
                    if as_bg {
                        termion::color::Rgb(r, g, b).bg_string()
                    } else {
                        termion::color::Rgb(r, g, b).fg_string()
                    }
                } else if as_bg {
                    termion::color::Rgb(0, 0, 0).bg_string()
                } else {
                    termion::color::Rgb(255, 255, 255).fg_string()
                }
            }
            _ => {
                if as_bg {
                    termion::color::Rgb(0, 0, 0).bg_string()
                } else {
                    termion::color::Rgb(192, 197, 206).fg_string()
                } // fallback
            }
        }
    }
}
