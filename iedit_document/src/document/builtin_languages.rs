use regex_lite::Regex;

use crate::{DocumentSyntax, SyntaxRule, document::syntax::parse_color_hex};

impl DocumentSyntax {
    pub fn builtin_python() -> Self {
        Self {
            name: "Python",
            rules: vec![
                // Comments
                SyntaxRule::Inline {
                    pattern: Regex::new(r"#.*$").unwrap(),
                    color: parse_color_hex("#6A9955", false).unwrap(),
                },
                SyntaxRule::Multiline {
                    start_pattern: Regex::new(r#"""""#).unwrap(),
                    end_pattern: Regex::new(r#"""""#).unwrap(),
                    color: parse_color_hex("#6A9955", false).unwrap(),
                },
                SyntaxRule::Multiline {
                    start_pattern: Regex::new(r#"'''"#).unwrap(),
                    end_pattern: Regex::new(r#"'''"#).unwrap(),
                    color: parse_color_hex("#6A9955", false).unwrap(),
                },
                // Strings
                SyntaxRule::Inline {
                    pattern: Regex::new(r#""(?:[^"\\]|\\.)*""#).unwrap(),
                    color: parse_color_hex("#CE9178", false).unwrap(),
                },
                SyntaxRule::Inline {
                    pattern: Regex::new(r#"'(?:[^'\\]|\\.)*'"#).unwrap(),
                    color: parse_color_hex("#CE9178", false).unwrap(),
                },
                // Keywords
                SyntaxRule::Inline {
                    pattern: Regex::new(r"\b(def|class|if|else|elif|for|while|return|import|from|as|try|except|finally|with|lambda|pass|break|continue|yield|async|await)\b").unwrap(),
                    color: parse_color_hex("#C586C0", false).unwrap(),
                },
                // Built-in functions and types
                SyntaxRule::Inline {
                    pattern: Regex::new(r"\b(print|len|str|int|float|list|dict|set|range|type|isinstance|None|True|False)\b").unwrap(),
                    color: parse_color_hex("#4EC9B0", false).unwrap(),
                },
                // Numbers
                SyntaxRule::Inline {
                    pattern: Regex::new(r"\b\d+\.?\d*\b").unwrap(),
                    color: parse_color_hex("#B5CEA8", false).unwrap(),
                },
                // Operators
                SyntaxRule::Inline {
                    pattern: Regex::new(r"[=+\-*/<>!&|]+").unwrap(),
                    color: parse_color_hex("#D4D4D4", false).unwrap(),
                },
            ],
        }
    }

    pub fn builtin_javascript() -> Self {
        Self {
            name: "JavaScript",
            rules: vec![
                // Single-line comments
                SyntaxRule::Inline {
                    pattern: Regex::new(r"//.*$").unwrap(),
                    color: parse_color_hex("#6A9955", false).unwrap(),
                },
                // Multi-line comments
                SyntaxRule::Multiline {
                    start_pattern: Regex::new(r"/\*").unwrap(),
                    end_pattern: Regex::new(r"\*/").unwrap(),
                    color: parse_color_hex("#6A9955", false).unwrap(),
                },
                // Template literals
                SyntaxRule::Inline {
                    pattern: Regex::new(r"`(?:[^`\\]|\\.)*`").unwrap(),
                    color: parse_color_hex("#CE9178", false).unwrap(),
                },
                // Strings
                SyntaxRule::Inline {
                    pattern: Regex::new(r#""(?:[^"\\]|\\.)*""#).unwrap(),
                    color: parse_color_hex("#CE9178", false).unwrap(),
                },
                SyntaxRule::Inline {
                    pattern: Regex::new(r#"'(?:[^'\\]|\\.)*'"#).unwrap(),
                    color: parse_color_hex("#CE9178", false).unwrap(),
                },
                // Keywords
                SyntaxRule::Inline {
                    pattern: Regex::new(r"\b(function|const|let|var|if|else|for|while|do|switch|case|break|continue|return|try|catch|finally|throw|new|delete|typeof|instanceof|this|super|class|extends|import|export|from|as|default|async|await|yield)\b").unwrap(),
                    color: parse_color_hex("#569CD6", false).unwrap(),
                },
                // Built-in objects and functions
                SyntaxRule::Inline {
                    pattern: Regex::new(r"\b(console|window|document|Array|Object|String|Number|Boolean|Math|Date|JSON|Promise|null|undefined|true|false)\b").unwrap(),
                    color: parse_color_hex("#4EC9B0", false).unwrap(),
                },
                // Numbers
                SyntaxRule::Inline {
                    pattern: Regex::new(r"\b\d+\.?\d*\b").unwrap(),
                    color: parse_color_hex("#B5CEA8", false).unwrap(),
                },
                // Operators
                SyntaxRule::Inline {
                    pattern: Regex::new(r"[=+\-*/<>!&|?:]+").unwrap(),
                    color: parse_color_hex("#D4D4D4", false).unwrap(),
                },
            ],
        }
    }

    pub fn builtin_rust() -> Self {
        Self {
            name: "Rust",
            rules: vec![
                // Single-line comments
                SyntaxRule::Inline {
                    pattern: Regex::new(r"//.*$").unwrap(),
                    color: parse_color_hex("#6A9955", false).unwrap(),
                },
                // Multi-line comments
                SyntaxRule::Multiline {
                    start_pattern: Regex::new(r"/\*").unwrap(),
                    end_pattern: Regex::new(r"\*/").unwrap(),
                    color: parse_color_hex("#6A9955", false).unwrap(),
                },
                // Raw strings
                SyntaxRule::Inline {
                    pattern: Regex::new(r##"r#".*?"#"##).unwrap(),
                    color: parse_color_hex("#CE9178", false).unwrap(),
                },
                // Strings
                SyntaxRule::Inline {
                    pattern: Regex::new(r#""(?:[^"\\]|\\.)*""#).unwrap(),
                    color: parse_color_hex("#CE9178", false).unwrap(),
                },
                // Character literals
                SyntaxRule::Inline {
                    pattern: Regex::new(r"'(?:[^'\\]|\\.)+'").unwrap(),
                    color: parse_color_hex("#CE9178", false).unwrap(),
                },
                // Keywords
                SyntaxRule::Inline {
                    pattern: Regex::new(r"\b(fn|let|mut|const|static|if|else|match|loop|while|for|in|break|continue|return|impl|trait|struct|enum|type|pub|mod|use|as|crate|super|self|where|unsafe|async|await|move|ref|dyn)\b").unwrap(),
                    color: parse_color_hex("#569CD6", false).unwrap(),
                },
                // Built-in types
                SyntaxRule::Inline {
                    pattern: Regex::new(r"\b(i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize|f32|f64|bool|char|str|String|Vec|Option|Result|Box|Rc|Arc|Some|None|Ok|Err|true|false)\b").unwrap(),
                    color: parse_color_hex("#4EC9B0", false).unwrap(),
                },
                // Macros
                SyntaxRule::Inline {
                    pattern: Regex::new(r"\b[a-z_][a-z0-9_]*!").unwrap(),
                    color: parse_color_hex("#DCDCAA", false).unwrap(),
                },
                // Lifetimes and labels
                SyntaxRule::Inline {
                    pattern: Regex::new(r"'[a-z_][a-z0-9_]*\b").unwrap(),
                    color: parse_color_hex("#4FC1FF", false).unwrap(),
                },
                // Numbers
                SyntaxRule::Inline {
                    pattern: Regex::new(r"\b\d+\.?\d*\b").unwrap(),
                    color: parse_color_hex("#B5CEA8", false).unwrap(),
                },
                // Operators
                SyntaxRule::Inline {
                    pattern: Regex::new(r"[=+\-*/<>!&|?:]+").unwrap(),
                    color: parse_color_hex("#D4D4D4", false).unwrap(),
                },
            ],
        }
    }

    pub fn builtin_bash() -> Self {
        Self {
            name: "Bash",
            rules: vec![
                // Comments
                SyntaxRule::Inline {
                    pattern: Regex::new(r"#.*$").unwrap(),
                    color: parse_color_hex("#6A9955", false).unwrap(),
                },
                // Double-quoted strings
                SyntaxRule::Inline {
                    pattern: Regex::new(r#""(?:[^"\\$]|\\.|\$[^(])*""#).unwrap(),
                    color: parse_color_hex("#CE9178", false).unwrap(),
                },
                // Single-quoted strings
                SyntaxRule::Inline {
                    pattern: Regex::new(r#"'(?:[^'\\]|\\.)*'"#).unwrap(),
                    color: parse_color_hex("#CE9178", false).unwrap(),
                },
                // Keywords
                SyntaxRule::Inline {
                    pattern: Regex::new(r"\b(if|then|else|elif|fi|case|esac|for|while|do|done|in|function|return|break|continue|exit|local|export|declare|readonly)\b").unwrap(),
                    color: parse_color_hex("#C586C0", false).unwrap(),
                },
                // Built-in commands
                SyntaxRule::Inline {
                    pattern: Regex::new(r"\b(echo|printf|read|cd|pwd|ls|cat|grep|sed|awk|find|sort|uniq|wc|head|tail|tr|cut|paste|test|source)\b").unwrap(),
                    color: parse_color_hex("#4EC9B0", false).unwrap(),
                },
                // Variables
                SyntaxRule::Inline {
                    pattern: Regex::new(r"\$[a-zA-Z_][a-zA-Z0-9_]*|\$\{[a-zA-Z_][a-zA-Z0-9_]*\}").unwrap(),
                    color: parse_color_hex("#9CDCFE", false).unwrap(),
                },
                // Special variables
                SyntaxRule::Inline {
                    pattern: Regex::new(r"\$[0-9@*#?$!-]").unwrap(),
                    color: parse_color_hex("#9CDCFE", false).unwrap(),
                },
                // Numbers
                SyntaxRule::Inline {
                    pattern: Regex::new(r"\b\d+\b").unwrap(),
                    color: parse_color_hex("#B5CEA8", false).unwrap(),
                },
                // Operators and redirects
                SyntaxRule::Inline {
                    pattern: Regex::new(r"[=+\-*/<>!&|;]+|^\d*[<>]&?\d*").unwrap(),
                    color: parse_color_hex("#D4D4D4", false).unwrap(),
                },
            ],
        }
    }

    pub fn builtin_c() -> Self {
        Self {
            name: "C",
            rules: vec![
                // Single-line comments
                SyntaxRule::Inline {
                    pattern: Regex::new(r"//.*$").unwrap(),
                    color: parse_color_hex("#6A9955", false).unwrap(),
                },
                // Multi-line comments
                SyntaxRule::Multiline {
                    start_pattern: Regex::new(r"/\*").unwrap(),
                    end_pattern: Regex::new(r"\*/").unwrap(),
                    color: parse_color_hex("#6A9955", false).unwrap(),
                },
                // Preprocessor directives
                SyntaxRule::Inline {
                    pattern: Regex::new(r"#[a-z]+").unwrap(),
                    color: parse_color_hex("#C586C0", false).unwrap(),
                },
                // Strings
                SyntaxRule::Inline {
                    pattern: Regex::new(r#""(?:[^"\\]|\\.)*""#).unwrap(),
                    color: parse_color_hex("#CE9178", false).unwrap(),
                },
                // Local Imports
                SyntaxRule::Inline {
                    pattern: Regex::new(r"<[^>]+>").unwrap(),
                    color: parse_color_hex("#CE9178", false).unwrap(),
                },
                // Character literals
                SyntaxRule::Inline {
                    pattern: Regex::new(r"'(?:[^'\\]|\\.)+'").unwrap(),
                    color: parse_color_hex("#CE9178", false).unwrap(),
                },
                // Keywords
                SyntaxRule::Inline {
                    pattern: Regex::new(r"\b(if|else|switch|case|default|for|while|do|break|continue|return|goto|typedef|struct|union|enum|sizeof|const|static|extern|auto|register|volatile|inline|restrict)\b").unwrap(),
                    color: parse_color_hex("#569CD6", false).unwrap(),
                },
                // Types
                SyntaxRule::Inline {
                    pattern: Regex::new(r"\b(void|char|short|int|long|float|double|signed|unsigned|size_t|ptrdiff_t|FILE|NULL)\b").unwrap(),
                    color: parse_color_hex("#4EC9B0", false).unwrap(),
                },
                // Numbers
                SyntaxRule::Inline {
                    pattern: Regex::new(r"\b\d+\.?\d*[fFlLuU]*\b|^0[xX][0-9a-fA-F]+").unwrap(),
                    color: parse_color_hex("#B5CEA8", false).unwrap(),
                },
                // Operators
                SyntaxRule::Inline {
                    pattern: Regex::new(r"[=+\-*/<>!&|?:~^%]+|^->|^\+\+|^--").unwrap(),
                    color: parse_color_hex("#D4D4D4", false).unwrap(),
                },
            ],
        }
    }

    pub fn builtin_cpp() -> Self {
        let mut cpp_syntax = Self::builtin_c();
        cpp_syntax.name = "C++";

        // Add C++-specific rules before the existing C rules
        let mut cpp_rules = vec![
            // Raw string literals (C++11)
            SyntaxRule::Inline {
                pattern: Regex::new(r#"R"\(.*?\)""#).unwrap(),
                color: parse_color_hex("#CE9178", false).unwrap(),
            },
            // C++ Keywords
            SyntaxRule::Inline {
                pattern: Regex::new(r"\b(class|namespace|template|typename|public|private|protected|virtual|override|final|constexpr|decltype|nullptr|new|delete|try|catch|throw|using|operator|friend|explicit|mutable|noexcept|static_cast|dynamic_cast|const_cast|reinterpret_cast)\b").unwrap(),
                color: parse_color_hex("#569CD6", false).unwrap(),
            },
            // C++ Types
            SyntaxRule::Inline {
                pattern: Regex::new(r"\b(bool|wchar_t|char16_t|char32_t|nullptr_t|true|false)\b").unwrap(),
                color: parse_color_hex("#4EC9B0", false).unwrap(),
            },
            // STL types
            SyntaxRule::Inline {
                pattern: Regex::new(r"\b(std|string|vector|map|set|list|deque|queue|stack|pair|tuple|array|unique_ptr|shared_ptr|weak_ptr)\b").unwrap(),
                color: parse_color_hex("#4EC9B0", false).unwrap(),
            },
        ];

        // Update operators to include C++ specific ones
        if let Some(op_idx) = cpp_syntax.rules.iter().position(|rule| {
            matches!(rule, SyntaxRule::Inline { pattern, .. } if pattern.as_str().contains("->"))
        }) {
            cpp_syntax.rules[op_idx] = SyntaxRule::Inline {
                pattern: Regex::new(r"[=+\-*/<>!&|?:~^%]+|^->|^\+\+|^--|^::").unwrap(),
                color: parse_color_hex("#D4D4D4", false).unwrap(),
            };
        }

        // Update numbers to include binary literals (C++14)
        if let Some(num_idx) = cpp_syntax.rules.iter().position(|rule| {
            matches!(rule, SyntaxRule::Inline { pattern, .. } if pattern.as_str().contains("0[xX]"))
        }) {
            cpp_syntax.rules[num_idx] = SyntaxRule::Inline {
                pattern: Regex::new(r"\b\d+\.?\d*[fFlLuU]*\b|^0[xX][0-9a-fA-F]+|^0[bB][01]+").unwrap(),
                color: parse_color_hex("#B5CEA8", false).unwrap(),
            };
        }

        // Prepend C++ specific rules
        cpp_rules.append(&mut cpp_syntax.rules);
        cpp_syntax.rules = cpp_rules;

        cpp_syntax
    }
}
