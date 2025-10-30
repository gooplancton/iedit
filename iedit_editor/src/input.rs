use crossbeam_channel::{Receiver, select, unbounded};
use std::fs;
use termion::{event::Key, input::TermRead};

#[non_exhaustive]
pub enum Input {
    NoOp,
    Keypress(Key),
    KeyChord([Key; 3]),
    ExternalNotification(String),
}

pub fn get_tty() -> fs::File {
    fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .expect("could not get tty")
}

pub struct InputParser {
    pub keys: Receiver<Key>,
    pub keychord_buf: [Key; 3],
    pub notifications: Receiver<String>,
}

impl InputParser {
    pub fn new(notifications: Receiver<String>) -> Self {
        let (sender, receiver) = unbounded();

        smol::spawn(async move {
            let tty = get_tty();
            let keys = tty.keys();

            for key in keys {
                if key.is_err() {
                    continue;
                }
                if sender.send(key.unwrap()).is_err() {
                    break;
                }
            }
        })
        .detach();

        Self {
            keys: receiver,
            keychord_buf: [Key::Null; 3],
            notifications,
        }
    }
}

impl Iterator for InputParser {
    type Item = Input;

    fn next(&mut self) -> Option<Self::Item> {
        select! {
            recv(self.notifications) -> msg => {
                msg.ok().map(Input::ExternalNotification)
            }
            recv(self.keys) -> key => {
                match key {
                    Ok(Key::Ctrl('k')) => {
                        self.keychord_buf = [Key::Null; 3];
                        self.keychord_buf[0] = Key::Ctrl('k');
                        Some(Input::NoOp)
                    }
                    Ok(key) if self.keychord_buf[1] != Key::Null => {
                        self.keychord_buf[2] = key;
                        let input = Input::KeyChord(self.keychord_buf);
                        self.keychord_buf = [Key::Null; 3];

                        Some(input)
                    },
                    Ok(key) if self.keychord_buf[0] != Key::Null => {
                        self.keychord_buf[1] = key;
                        Some(Input::NoOp)
                    },
                    Ok(key) => Some(Input::Keypress(key)),
                    Err(_) => Some(Input::NoOp),
                }
            }
        }
    }
}
