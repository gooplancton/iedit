use std::io::{Seek, SeekFrom, Write};

use super::Editor;

impl Editor {
    pub fn save_file(&mut self) -> std::io::Result<()> {
        // naive implemetation for now, in the future we can keep track of
        // lines that have been modified and only write them
        self.file.set_len(0);
        self.file.seek(SeekFrom::Start(0))?;
        let mut file_writer = std::io::BufWriter::new(&self.file);
        self.file_lines
            .iter()
            .enumerate()
            .try_for_each(|(line_idx, line)| {
                line.iter()
                    .try_for_each(|char| write!(file_writer, "{}", char))
                    .and_then(|_| {
                        if line_idx != self.file_lines.len() - 1 {
                            write!(file_writer, "\n")
                        } else {
                            Ok(())
                        }
                    })
            })?;

        file_writer.flush()?;

        self.state.is_file_modified = false;

        Ok(())
    }
}
