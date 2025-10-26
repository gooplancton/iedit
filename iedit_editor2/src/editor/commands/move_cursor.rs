pub enum MoveCursor {
    ToAbsolutePos((usize, usize)),
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
    NextWord,
    PreviousWord,
    NextParagraph,
    PreviousParagraph,
    MatchingParenthesis,
    NextOccurrenceOf(char),
    PreviousOccurrenceOf(char),
}
