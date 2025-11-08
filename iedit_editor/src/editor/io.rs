use std::{
    fs::File,
    io::{Seek, SeekFrom, Write},
    path::PathBuf,
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

        // naive implemetation for now, in the future we can keep track of
        // lines that have been modified and only write them
        let n_lines = self.document.n_lines();
        if let Some(file) = &mut self.document.file {
            file.set_len(0)?;
            file.seek(SeekFrom::Start(0))?;
            let mut file_writer = std::io::BufWriter::new(file);
            self.document
                .lines
                .iter()
                .enumerate()
                .try_for_each(|(line_idx, line)| {
                    write!(file_writer, "{}", line.as_ref())?;
                    if line_idx != n_lines - 1 {
                        writeln!(file_writer)
                    } else {
                        Ok(())
                    }
                })?;

            file_writer.flush()?;

            self.document.has_been_edited = false;
            if display_notification {
                send_simple_notification("File saved");
            }
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

        // TODO: handle this
        let _ = self.save_file(false);

        CommandExecutionResult::Continue
    }
}
