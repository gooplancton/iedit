#![allow(warnings)]

use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Read, Stdout, Write, stdin},
    path::Path,
    process::exit,
};

use config::EditorConfig;
use state::EditorState;
use termion::{
    cursor::{DetectCursorPos, Goto, HideCursor},
    raw::{IntoRawMode, RawTerminal},
    terminal_size,
};

use crate::{
    editor::input::{EditorInput, InputReader},
    terminal::{CURSOR_DOWN1, H_BAR},
};

mod command;
mod config;
mod cursor;
mod edit;
mod input;
mod io;
mod render;
mod state;
mod viewport;

pub struct Editor {
    file: File,
    file_name: String,
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
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .append(false)
            .create(true)
            .open(path.as_ref())?;

        if file.metadata()?.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Provided path is a directory",
            ));
        }

        let file_name = path.as_ref().components().last().unwrap().as_os_str();
        let mut file_lines = Vec::<String>::new();
        let mut file_reader = BufReader::new(file);
        let mut file_line = String::default();
        while file_reader.read_line(&mut file_line)? > 0 {
            file_lines.push(file_line.trim_end_matches("\n").to_string());
            file_line.truncate(0);
        }

        if file_lines.is_empty() {
            file_lines.push(String::new());
        }

        let file = file_reader.into_inner();
        let mut state = EditorState::default();

        let (term_width, term_height) = terminal_size()?;
        let mut term = HideCursor::from(std::io::stdout().into_raw_mode()?);
        let mut ui_start = term.cursor_pos()?;

        let mut config = EditorConfig::default();
        if config.n_lines == 0 {
            let offset = config.set_default_n_lines(&mut term, ui_start.1)?;
            ui_start.1 = ui_start.1.saturating_sub(offset);
        }

        state.viewport.bottom_line = config.n_lines as usize;
        state.viewport.right_col = (term_width as usize) - (config.show_line_numbers as usize * 7);

        let horizontal_bar = str::repeat(H_BAR, term_width as usize);

        Ok(Self {
            file,
            file_name: file_name.to_string_lossy().to_string(),
            file_lines,
            state,
            config,
            term,
            term_width,
            reset_cursor_seq: Goto(ui_start.0, ui_start.1),
            horizontal_bar,
        })
    }

    pub fn quit(&mut self) -> std::io::Result<()> {
        self.cleanup()?;

        exit(0);
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        self.render()?;

        let mut stdin = stdin();
        loop {
            let prev_x = self.state.cursor_pos_x as isize;
            let prev_y = self.state.cursor_pos_y as isize;

            let input = stdin.get_input()?;
            if input.should_quit() {
                break;
            }

            self.process_input(input)?;

            self.clamp_cursor();

            self.state.cursor_vel_x = self.state.cursor_pos_x as isize - prev_x;
            self.state.cursor_vel_y = self.state.cursor_pos_y as isize - prev_y;

            self.adjust_viewport();

            self.render()?;
        }

        self.cleanup()?;
        Ok(())
    }
}
