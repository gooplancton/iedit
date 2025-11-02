use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    thread::{JoinHandle, spawn},
};

use iedit_document::Document;
use termion::event::Key;

use crate::{
    Editor,
    editor::{FILE_EXECUTION_OUTPUT, commands::notify::send_notification},
};

impl Editor {
    pub fn execute_file(&mut self, executor_key: Key) {
        if let Err(_) = self.save_file(false) {
            send_notification("Could not save file for execution");
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

        let _handle: JoinHandle<()> = spawn(move || {
            if let Err(e) = run_command(&command) {
                send_notification(format!("Error executing command: {}", e));
            }
        });
    }

    fn get_executor_by_key(&self, executor_key: Key) -> Option<&str> {
        match executor_key {
            Key::Char('p') => Some("/usr/bin/env python3"),
            Key::Char('P') => Some("/usr/bin/env python"),
            Key::Char('n') => Some("/usr/bin/env node"),
            Key::Char('b') => Some("/usr/bin/env bash"),
            Key::Char('x') => {
                let shebang_line = self.document.lines.first()?;
                if shebang_line.starts_with("#!") {
                    return shebang_line.get(2..);
                }

                None
            }
            _ => None,
        }
    }
}

fn run_command(command: &str) -> std::io::Result<()> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| std::io::Error::other("Failed to capture stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| std::io::Error::other("Failed to capture stderr"))?;

    send_notification(format!("Executing: {}", command));

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let mut output_lines = vec![];

    // Process stdout and stderr concurrently
    output_lines.push("// STDOUT //".to_owned());
    for line in stdout_reader.lines() {
        if let Ok(line) = line {
            output_lines.push(line);
        }
    }

    output_lines.push("// STDERR //".to_owned());
    for line in stderr_reader.lines() {
        if let Ok(line) = line {
            output_lines.push(line);
        }
    }

    let output = Document::new(output_lines);

    if let Ok(mut file_execution_output) = FILE_EXECUTION_OUTPUT.lock()
        && file_execution_output.is_none()
    {
        *file_execution_output = Some(output)
    }

    // Wait for the process to complete
    let status = child.wait()?;
    send_notification(format!(
        "Process exited with status: {}, To view output: Ctrl+k v o",
        status
    ));

    Ok(())
}
