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
}

#[derive(Default)]
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
        });

        // Types - soft cyan
        rules.push(SyntaxRule {
            pattern: Regex::new(r"\b(i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize|f32|f64|bool|char|str|String|Vec|Option|Result|Box)\b").unwrap(),
            color: termion::color::Rgb(150, 181, 180).fg_string(),
        });

        // Comments - muted gray
        rules.push(SyntaxRule {
            pattern: Regex::new(r"//.*$|/\*(?:[^*]|\*[^/])*\*/").unwrap(),
            color: termion::color::Rgb(101, 115, 126).fg_string(),
        });

        // Strings - soft orange
        rules.push(SyntaxRule {
            pattern: Regex::new(r#""([^"\\]|\\.)*"|'([^'\\]|\\.)*'"#).unwrap(),
            color: termion::color::Rgb(208, 135, 112).fg_string(),
        });

        // Numbers - soft purple
        rules.push(SyntaxRule {
            pattern: Regex::new(r"\b\d[\d_]*(\.\d[\d_]*)?([eE][+-]?\d[\d_]*)?\b").unwrap(),
            color: termion::color::Rgb(180, 142, 173).fg_string(),
        });

        Self { rules }
    }

    fn builtin_python() -> Self {
        let mut rules = Vec::new();

        // Keywords - steel blue
        rules.push(SyntaxRule {
            pattern: Regex::new(r"\b(def|class|if|else|elif|for|while|return|import|from|as|try|except|finally|with|lambda|yield|raise|break|continue|pass|assert|del|global|nonlocal|in|is|not|and|or)\b").unwrap(),
            color: termion::color::Rgb(143, 161, 179).fg_string(),
        });

        // Built-in functions and types - soft cyan
        rules.push(SyntaxRule {
            pattern: Regex::new(r"\b(print|len|str|int|float|list|dict|set|tuple|bool|range|enumerate|zip|map|filter|any|all|sum|min|max|sorted|reversed|iter|next|super|isinstance|hasattr|getattr|setattr)\b").unwrap(),
            color: termion::color::Rgb(150, 181, 180).fg_string(),
        });

        // Comments - muted gray
        rules.push(SyntaxRule {
            pattern: Regex::new(r"#.*$").unwrap(),
            color: termion::color::Rgb(101, 115, 126).fg_string(),
        });

        // Strings - soft orange
        rules.push(SyntaxRule {
            pattern: Regex::new(
                r#"("""[\s\S]*?""")|'''[\s\S]*?'''|"([^"\\]|\\.)*"|'([^'\\]|\\.)*'"#,
            )
            .unwrap(),
            color: termion::color::Rgb(208, 135, 112).fg_string(),
        });

        Self { rules }
    }

    fn builtin_javascript() -> Self {
        let mut rules = Vec::new();

        // Keywords - steel blue
        rules.push(SyntaxRule {
            pattern: Regex::new(r"\b(const|let|var|function|class|if|else|for|while|do|switch|case|break|continue|return|try|catch|finally|throw|new|delete|typeof|instanceof|void|yield|async|await|of|in)\b").unwrap(),
            color: termion::color::Rgb(143, 161, 179).fg_string(),
        });

        // Built-in objects and functions - soft cyan
        rules.push(SyntaxRule {
            pattern: Regex::new(r"\b(Array|Object|String|Number|Boolean|Function|RegExp|Date|Promise|Map|Set|Symbol|console|Math|JSON)\b").unwrap(),
            color: termion::color::Rgb(150, 181, 180).fg_string(),
        });

        // Comments - muted gray
        rules.push(SyntaxRule {
            pattern: Regex::new(r"//.*$|/\*(?:[^*]|\*[^/])*\*/").unwrap(),
            color: termion::color::Rgb(101, 115, 126).fg_string(),
        });

        // Strings - soft orange
        rules.push(SyntaxRule {
            pattern: Regex::new(r#""([^"\\]|\\.)*"|'([^'\\]|\\.)*'|`([^`\\]|\\.)*`"#).unwrap(),
            color: termion::color::Rgb(208, 135, 112).fg_string(),
        });

        Self { rules }
    }

    pub fn infer_from_extension(ext: &std::ffi::OsStr, base_path: Option<PathBuf>) -> Option<Self> {
        if let Some(base_path) = base_path {
            if let Some(syntax_highlighter) = Self::load_from_nanorc(
                base_path
                    .join(format!("{}.nanorc", ext.to_str()?))
                    .as_path(),
            )
            .ok()
            {
                return Some(syntax_highlighter);
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
        let mut current_color = None;

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            if let Some(color_str) = line.strip_prefix("color ") {
                let parts: Vec<&str> = color_str.split_whitespace().collect();
                if parts.len() >= 2 {
                    current_color = Some(Self::parse_color(parts[0]));
                }
            } else if let Some(pattern) = line.strip_prefix("icolor ") {
                if let Some(ref color) = current_color {
                    if let Ok(regex) = Self::convert_nano_regex(pattern) {
                        highlighter.rules.push(SyntaxRule {
                            pattern: regex,
                            color: color.clone(),
                        });
                    }
                }
            }
        }

        Ok(highlighter)
    }

    fn parse_color(color: &str) -> String {
        // Using base16-ocean.dark inspired colors
        match color.to_lowercase().as_str() {
            "red" => termion::color::Rgb(191, 97, 106).fg_string(), // Soft red
            "green" => termion::color::Rgb(163, 190, 140).fg_string(), // Soft green
            "blue" => termion::color::Rgb(143, 161, 179).fg_string(), // Steel blue
            "yellow" => termion::color::Rgb(235, 203, 139).fg_string(), // Soft yellow
            "magenta" => termion::color::Rgb(180, 142, 173).fg_string(), // Soft purple
            "cyan" => termion::color::Rgb(150, 181, 180).fg_string(), // Soft cyan
            "orange" => termion::color::Rgb(208, 135, 112).fg_string(), // Soft orange
            // Support for RGB format remains the same
            _ if color.starts_with('#') && color.len() == 7 => {
                if let Ok(r) = u8::from_str_radix(&color[1..3], 16) {
                    if let Ok(g) = u8::from_str_radix(&color[3..5], 16) {
                        if let Ok(b) = u8::from_str_radix(&color[5..7], 16) {
                            return termion::color::Rgb(r, g, b).fg_string();
                        }
                    }
                }
                termion::color::LightWhite.fg_str().to_owned() // fallback to white
            }
            _ => termion::color::Rgb(192, 197, 206).fg_string(), // Light gray as fallback
        }
    }

    fn convert_nano_regex(pattern: &str) -> Result<Regex, regex_lite::Error> {
        // Convert nano regex syntax to Rust regex syntax
        // This is a basic conversion - extend as needed
        let mut rust_pattern = pattern.to_string();

        // Replace nano's \< and \> with Rust's \b
        rust_pattern = rust_pattern.replace(r"\<", r"\b");
        rust_pattern = rust_pattern.replace(r"\>", r"\b");

        Regex::new(&rust_pattern)
    }
}
