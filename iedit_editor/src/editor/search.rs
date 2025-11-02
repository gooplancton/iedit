use regex_lite::Regex;

pub enum SearchItem {
    Regex(Regex),
    PromptString,
    DocumentRange {
        pos_from: (usize, usize),
        pos_to: (usize, usize),
    },
}


