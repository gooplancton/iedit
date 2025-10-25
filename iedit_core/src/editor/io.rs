use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use crate::line::EditorLine;

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
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
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
        // naive implemetation for now, in the future we can keep track of
        // lines that have been modified and only write them
        if self.file.is_none() {
            self.file = Some(File::create_new(self.canonicalized_file_path.as_path())?);
        }

        if let Some(file) = &mut self.file {
            file.set_len(0)?;
            file.seek(SeekFrom::Start(0))?;
            let mut file_writer = std::io::BufWriter::new(file);
            self.file_lines
                .iter()
                .enumerate()
                .try_for_each(|(line_idx, line)| {
                    write!(file_writer, "{}", line)?;
                    if line_idx != self.file_lines.len() - 1 {
                        writeln!(file_writer)
                    } else {
                        Ok(())
                    }
                })?;

            file_writer.flush()?;

            self.state.is_file_modified = false;
            self.temp_message = "File saved".to_owned();
        }

        Ok(())
    }
}
