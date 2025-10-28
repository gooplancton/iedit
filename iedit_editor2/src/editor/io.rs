use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use crate::{editor::commands::CommandExecutionResult, line::DocumentLine};

use super::Editor;

type ReadFile = (Option<File>, PathBuf, Vec<String>);

pub fn read_file(path: impl AsRef<Path>) -> io::Result<ReadFile> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .append(false)
        .open(path.as_ref())
        .and_then(|file| {
            if file.metadata()?.is_dir() {
                Err(io::Error::from(io::ErrorKind::IsADirectory))
            } else {
                Ok(file)
            }
        });

    match file {
        Err(err)
            if path.as_ref().as_os_str().is_empty() || err.kind() == io::ErrorKind::NotFound =>
        {
            Ok((None, path.as_ref().to_owned(), Vec::new()))
        }
        Err(err) => Err(err),
        Ok(file) => {
            let mut file_lines = Vec::new();
            let mut file_reader = BufReader::new(file);
            let mut file_line = String::default();
            while file_reader.read_line(&mut file_line)? > 0 {
                file_lines.push(String::from_str_trim_newline(&file_line));
                file_line.truncate(0);
            }

            let file = file_reader.into_inner();
            Ok((Some(file), path.as_ref().to_owned(), file_lines))
        }
    }
}

impl Editor {
    pub fn save_file(&mut self) -> std::io::Result<()> {
        if self.file.is_none() && self.canonicalized_file_path.as_os_str().is_empty() {
            self.prompt_user("File name: ", Editor::set_file);

            return Ok(());
        } else if self.file.is_none() {
            self.file = Some(File::create_new(&self.canonicalized_file_path)?);
        }

        // naive implemetation for now, in the future we can keep track of
        // lines that have been modified and only write them
        if let Some(file) = &mut self.file {
            file.set_len(0)?;
            file.seek(SeekFrom::Start(0))?;
            let mut file_writer = std::io::BufWriter::new(file);
            self.document
                .lines
                .iter()
                .enumerate()
                .try_for_each(|(line_idx, line)| {
                    write!(file_writer, "{}", line)?;
                    if line_idx != self.document.n_lines() - 1 {
                        writeln!(file_writer)
                    } else {
                        Ok(())
                    }
                })?;

            file_writer.flush()?;

            self.document.has_been_edited = false;
            self.status_bar.update_notification("File saved");
        }

        Ok(())
    }

    pub fn set_file(&mut self, path: String) -> CommandExecutionResult {
        let canonicalized_file_path = if path.starts_with("/") {
            if let Ok(p) = PathBuf::from(path).canonicalize() {
                p
            } else {
                return CommandExecutionResult::Continue;
            }
        } else {
            if let Ok(mut p) = std::env::current_dir() {
                p.push(path);
                p
            } else {
                return CommandExecutionResult::Continue;
            }
        };

        if let Ok(file) = File::create_new(&canonicalized_file_path) {
            self.file = Some(file);
            self.canonicalized_file_path = canonicalized_file_path;
        }

        self.save_file();

        CommandExecutionResult::Continue
    }
}
