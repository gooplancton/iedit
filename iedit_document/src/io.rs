use std::{fs::{File, OpenOptions}, io::{self, BufRead, BufReader}, path::{Path, PathBuf}};

use crate::DocumentLine;

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
