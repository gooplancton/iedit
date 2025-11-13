use std::{
    fs::File,
    io::{Seek, SeekFrom, Write},
    path::PathBuf,
    time::SystemTime,
};

use iedit_document::DocumentLine;

use crate::editor::commands::{CommandExecutionResult, send_simple_notification};

use super::Editor;

impl Editor {
    pub fn save_file(&mut self, display_notification: bool) -> std::io::Result<()> {
        if self.is_viewing_execution_output {
            send_simple_notification("Currently viewing execution output, won't save");
            return Ok(());
        }

        if self.document.file.is_none()
            && self.document.canonicalized_file_path.as_os_str().is_empty()
        {
            self.prompt_user("File name: ", Editor::set_file);

            return Ok(());
        } else if self.document.file.is_none() {
            self.document.file = Some(File::create_new(&self.document.canonicalized_file_path)?);
        }

        let mut bytes_written = 0;
        let n_lines = self.document.n_lines();
        let file = self.document.file.as_mut().unwrap();
        file.seek(SeekFrom::Start(0))?;

        let is_untouched = file.metadata().is_ok_and(|metadata| {
            metadata
                .modified()
                .is_ok_and(|modified| modified <= self.document.last_save_time)
        });

        let first_modified_line_idx = if is_untouched {
            let first_modified_line_idx = self
                .document
                .lines
                .iter()
                .position(|line| line.has_been_modified)
                .unwrap_or(n_lines);

            if let Some(new_file_len) = self.document.line_offsets.get(first_modified_line_idx) {
                let new_file_len = *new_file_len;
                file.set_len(new_file_len)?;
                file.seek(SeekFrom::End(0))?;
            } else {
                file.seek(SeekFrom::End(0))?;
                if !self.document.line_offsets.is_empty() {
                    file.write_all(&[b'\n'])?;
                }
            }

            first_modified_line_idx
        } else {
            // NOTE: file has been modified outiside of iedit, present user with options:
            // - write to different file
            // - overwrite
            // - reload file
            file.set_len(0)?;
            0
        };

        let mut new_offsets = Vec::with_capacity(n_lines - first_modified_line_idx);
        let mut file_writer = std::io::BufWriter::new(file);
        let mut last_offset = file_writer.stream_position()?;
        for (line_idx, line) in self
            .document
            .lines
            .iter_mut()
            .enumerate()
            .skip(first_modified_line_idx)
        {
            line.has_been_modified = false;

            let text = line.as_ref();
            write!(file_writer, "{}", text)?;
            let mut line_bytes = text.as_bytes().len();
            if line_idx != n_lines - 1 {
                writeln!(file_writer)?;
                line_bytes += 1;
            }

            new_offsets.push(last_offset);
            bytes_written += line_bytes;
            last_offset += line_bytes as u64;
        }

        file_writer.flush()?;

        self.document.last_save_time = SystemTime::now();
        self.document.line_offsets.truncate(first_modified_line_idx);
        self.document.line_offsets.extend(new_offsets);

        if display_notification {
            send_simple_notification(format!("File saved. {} bytes written", bytes_written));
        }

        Ok(())
    }

    pub fn set_file(&mut self, path: DocumentLine) -> CommandExecutionResult {
        let canonicalized_file_path = if path.starts_with("/") {
            if let Ok(p) = PathBuf::from(path.as_ref()).canonicalize() {
                p
            } else {
                return CommandExecutionResult::Continue;
            }
        } else if let Ok(mut p) = std::env::current_dir() {
            p.push(path.as_ref());
            p
        } else {
            return CommandExecutionResult::Continue;
        };

        if let Ok(file) = File::create_new(&canonicalized_file_path) {
            self.document.file = Some(file);
            self.document.canonicalized_file_path = canonicalized_file_path;
        }

        // TODO: handle error here
        let _ = self.save_file(false);

        CommandExecutionResult::Continue
    }
}
