use std::io::Write;

use super::Editor;

impl Editor {
    fn save_file(&mut self) -> std::io::Result<()> {
        // naive implemetation for now, in the future we can keep track of
        // lines that have been modified and only write them

        self.file_lines
            .iter()
            .try_for_each(|line| self.file_writer.write_all(line.as_str().as_bytes()))
    }
}
