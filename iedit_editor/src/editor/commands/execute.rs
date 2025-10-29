use termion::event::Key;

use crate::Editor;

impl Editor {
    pub fn execute_file(&mut self, executor_key: Key) {
        let executor = self.get_executor_by_key(executor_key);
        if executor.is_none() {
            return;
        }

        let executor = executor.unwrap();

    }

    fn get_executor_by_key(&self, executor_key: Key) -> Option<&str> {
        match executor_key {
            Key::Char('p') => Some("/usr/bin/env python3"),
            Key::Char('P') => Some("/usr/bin/env python"),
            Key::Char('n') => Some("/usr/bin/env node"),
            Key::Char('b') => Some("/usr/bin/env bash"),
            Key::Char('x') => {
                let shbang_line = self.document.lines.first()?;
                if shbang_line.starts_with("#!") {
                    return shbang_line.get(2..)
                }

                None
            }
            _ => None
        }
    }
}
