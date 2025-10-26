pub enum Edit {
    InsertCharAt {
        pos: (usize, usize),
        ch: char,
    },
    DeleteCharAt {
        pos: (usize, usize),
    },
    InsertStringAt {
        pos: (usize, usize),
        string: String,
    },
    DeletePreviousWord {
        pos: (usize, usize),
    },
    DeleteRange {
        pos_from: (usize, usize),
        pos_to: (usize, usize),
    },
    ReplaceRange {
        pos_from: (usize, usize),
        pos_to: (usize, usize),
        string: String,
    },
    InsertStringsAtMultiline {
        pos: (usize, usize),
        strings: Vec<String>,
    },
    ReplaceRangeMultiline {
        pos_from: (usize, usize),
        pos_to: (usize, usize),
        strings: Vec<String>,
    },
}
