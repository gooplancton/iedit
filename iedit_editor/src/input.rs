use std::io::Stdin;

use termion::{event::Key, input::Keys};

#[non_exhaustive]
pub enum Input {
    NoOp,
    Keypress(Key),
    KeyChord([Key; 3]),
    TerminalWindowResize,
    ExternalNotification(String),
}

pub struct InputParser {
    pub keys: Keys<Stdin>,
}

impl Iterator for InputParser {
    type Item = Input;

    fn next(&mut self) -> Option<Self::Item> {
        match self.keys.next()?.ok()? {
            // TODO: parse ctrl + shift + arrows chords
            Key::Ctrl('k') => {
                let second_key = self.keys.next()?.unwrap_or(Key::End);
                if second_key == Key::Esc {
                    return Some(Input::NoOp);
                }
                let third_key = self.keys.next()?.unwrap_or(Key::End);
                if second_key == Key::Esc {
                    return Some(Input::NoOp);
                }

                Some(Input::KeyChord([Key::Ctrl('k'), second_key, third_key]))
            }
            key => Some(Input::Keypress(key)),
        }
    }
}
