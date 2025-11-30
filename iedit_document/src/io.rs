use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};

use crate::line::DocumentLine;

type ReadFile = (Option<File>, PathBuf, Vec<DocumentLine>, Vec<u64>, String);

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
            Ok((
                None,
                path.as_ref().to_owned(),
                Vec::new(),
                Vec::new(),
                "\n".to_owned(),
            ))
        }
        Err(err) => Err(err),
        Ok(file) => {
            let mut file_lines = Vec::new();
            let mut file_reader = BufReader::new(file);
            let mut file_line = String::default();
            let mut line_offsets = vec![];
            let mut last_offset = 0;
            let mut is_last_line_newline_terminated = false;
            let mut end_of_line_seq: Option<String> = None;
            while file_reader.read_line(&mut file_line)? > 0 {
                let bytes_read = file_line.len();

                let trimmed = if let Some(end_of_line_seq) = &end_of_line_seq {
                    file_line.trim_end_matches(end_of_line_seq)
                } else {
                    let trimmed = file_line.trim_end_matches(['\n', '\r']);
                    end_of_line_seq = Some(file_line[trimmed.len()..bytes_read].to_owned());

                    trimmed
                };

                is_last_line_newline_terminated = trimmed.len() < bytes_read;
                let mut line = DocumentLine::new(trimmed.to_string());
                line.has_been_modified = false;
                file_lines.push(line);
                file_line.truncate(0);

                line_offsets.push(last_offset);
                last_offset += bytes_read as u64;
            }

            if is_last_line_newline_terminated {
                let mut line = DocumentLine::default();
                line.has_been_modified = false;
                file_lines.push(line);
                line_offsets.push(last_offset);
            }

            let file = file_reader.into_inner();
            Ok((
                Some(file),
                path.as_ref().to_owned(),
                file_lines,
                line_offsets,
                end_of_line_seq.unwrap_or("\n".to_string()),
            ))
        }
    }
}
