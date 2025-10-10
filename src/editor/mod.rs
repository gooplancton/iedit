use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Stdout, stdin},
    path::Path,
};

use config::EditorConfig;
use state::EditorState;
use termion::{
    cursor::{DetectCursorPos, Goto, HideCursor},
    raw::{IntoRawMode, RawTerminal},
    terminal_size,
};

use crate::terminal::H_BAR;

mod command;
mod config;
mod input;
mod io;
mod movement;
mod render;
mod state;

pub struct Editor {
    file_writer: BufWriter<File>,
    file_lines: Vec<String>,
    state: EditorState,
    config: EditorConfig,
    term: HideCursor<RawTerminal<Stdout>>,
    term_width: u16,

    reset_cursor_seq: Goto,
    horizontal_bar: String,
}

impl Editor {
    pub fn new(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let file = File::open(path)?;

        let mut file_lines = Vec::<String>::new();
        let mut file_reader = BufReader::new(file);
        let mut file_line = String::default();
        while file_reader.read_line(&mut file_line)? > 0 {
            file_lines.push(file_line.trim_end_matches("\n").to_string());
            file_line.truncate(0);
        }

        let file = file_reader.into_inner();
        let file_writer = BufWriter::new(file);

        let state = EditorState::default();
        let mut config = EditorConfig {
            show_line_numbers: true,
            ..Default::default()
        };

        config.show_line_numbers = true;

        let (term_width, term_height) = terminal_size()?;

        if config.n_lines == 0 {
            config.n_lines = term_height / 2;
        }

        let mut term = HideCursor::from(std::io::stdout().into_raw_mode()?);

        let ui_start = term.cursor_pos()?;
        let reset_cursor_seq = Goto(ui_start.0, ui_start.1);

        let horizontal_bar = str::repeat(H_BAR, term_width as usize);

        Ok(Self {
            file_writer,
            file_lines,
            state,
            config,
            term,
            term_width,
            reset_cursor_seq,
            horizontal_bar,
        })
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        self.render()?;
        let mut stdin = stdin();
        let mut input_buf = [0_u8];
        loop {
            let _ = stdin.read_exact(&mut input_buf);
            let input_char = input_buf[0];
            let prev_x = self.state.cursor_pos_x;
            let prev_y = self.state.cursor_vel_y;
            match input_char as char {
                'q' => break,
                'j' => self.move_cursor_down(),
                'k' => self.move_cursor_up(),
                'l' => self.move_cursor_right(),
                'h' => self.move_cursor_left(),
                _ => continue,
            }

            self.clamp_cursor();

            self.state.cursor_vel_x = self.state.cursor_pos_x - prev_x;
            self.state.cursor_vel_y = self.state.cursor_pos_y - prev_y;

            self.render()?;
        }

        self.cleanup()?;
        Ok(())
    }
}
