use std::io::Stdin;

use termion::{event::Key, input::Keys};

#[non_exhaustive]
pub enum Input {
    Keypress(Key),
    KeyChord([Key; 3]),
    TerminalWindowResize,
}

pub struct InputParser {
    pub keys: Keys<Stdin>,
    pub chord_buffer: [Option<Key>; 3],
}

impl Iterator for InputParser {
    type Item = Input;

    fn next(&mut self) -> Option<Self::Item> {
        match self.keys.next()?.ok()? {
            // TODO: parse ctrl + shift + arrows chords
            key => Some(Input::Keypress(key)),
        }
    }
}
