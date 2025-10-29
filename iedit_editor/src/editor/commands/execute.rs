use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    thread::spawn,
};

use termion::event::Key;

use crate::{
    Editor,
    editor::{NOTIFICATION_SENDER, commands::notify::send_notification},
};

impl Editor {
    pub fn execute_file(&mut self, executor_key: Key) {
        if let Err(err) = self.save_file(false) {
            self.status_bar
                .update_notification("Could not save file for execution");
            return;
        };

        let executor = self.get_executor_by_key(executor_key);
        if executor.is_none() {
            return;
        }

        let executor = executor.unwrap();
        let command = format!(
            "{} {}",
            executor,
            self.canonicalized_file_path.as_path().display()
        );

        self.is_executing_file = true;

        spawn(move || {
            let mut child = Command::new("sh")
                .arg("-c")
                .arg(&command)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to execute command");

            let stdout = child.stdout.take().expect("Failed to capture stdout");
            let stderr = child.stderr.take().expect("Failed to capture stderr");

            let stdout_reader = BufReader::new(stdout);
            let stderr_reader = BufReader::new(stderr);

            // Send command as first notification
            send_notification(format!("Executing: {}", command));

            // Read and send stdout
            for line in stdout_reader.lines() {
                if let Ok(line) = line {
                    send_notification(format!("stdout: {}", line));
                }
            }

            // Read and send stderr
            for line in stderr_reader.lines() {
                if let Ok(line) = line {
                    send_notification(format!("stderr: {}", line));
                }
            }

            // Send exit status
            let status = child.wait().expect("Failed to wait for child process");
            // let _ = sender.send(format!("Process exited with status: {}", status));
        });
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
                    return shbang_line.get(2..);
                }

                None
            }
            _ => None,
        }
    }
}
