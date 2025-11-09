use regex_lite::Regex;

pub enum SearchItem {
    Regex(Regex),
    PromptString,
}
