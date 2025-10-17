use std::{
    fs::File,
    io::{Stdout, stdin},
    path::{Path, PathBuf},
};

use config::EditorConfig;
use state::EditorState;
use termion::{
    cursor::{DetectCursorPos, Goto, HideCursor},
    raw::{IntoRawMode, RawTerminal},
    terminal_size,
};

use crate::{
    editor::{input::InputReader, io::read_file},
    line::EditorLine,
    terminal::H_BAR,
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

pub struct Editor<TextLine: EditorLine> {
    file: Option<File>,
    canonicalized_file_path: PathBuf,
    file_lines: Vec<TextLine>,
    state: EditorState<TextLine>,
    config: EditorConfig,
    term: HideCursor<RawTerminal<Stdout>>,
    term_width: u16,

    reset_cursor_seq: Goto,
    horizontal_bar: String,
}

impl<TextLine: EditorLine> Editor<TextLine> {
    pub fn new(path: impl AsRef<Path>, open_at: usize) -> std::io::Result<Self> {
        let (file, canonicalized_file_path, file_lines) = read_file(path)?;
        let mut state = EditorState::default();

        let (term_width, term_height) = terminal_size()?;
        let mut term = HideCursor::from(std::io::stdout().into_raw_mode()?);
        let mut ui_start = term.cursor_pos()?;

        let mut config = if let Some(mut path) = std::env::home_dir() {
            path.push(".iedit.conf");
            EditorConfig::from_file(path).unwrap_or_default()
        } else {
            EditorConfig::default()
        };

        let real_estate = term_height.saturating_sub(ui_start.1);
        if config.n_lines == 0 || real_estate < config.n_lines {
            let offset = config.set_default_n_lines(&mut term, ui_start.1)?;
            ui_start.1 = ui_start.1.saturating_sub(offset);
        }

        state.cursor_pos_y = open_at;
        state.viewport.top_line = open_at.saturating_sub(config.n_lines as usize / 2);
        state.viewport.bottom_line = state.viewport.top_line + config.n_lines as usize;
        state.viewport.right_col = (term_width as usize) - (config.show_line_numbers as usize * 7);

        let horizontal_bar = str::repeat(H_BAR, term_width as usize);

        Ok(Self {
            file,
            canonicalized_file_path,
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
        self.state.should_quit = true;
        Ok(())
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        self.render()?;

        let mut stdin = stdin();
        loop {
            let prev_x = self.state.cursor_pos_x as isize;
            let prev_y = self.state.cursor_pos_y as isize;

            let input = stdin.get_input()?;
            self.process_input(input)?;

            if self.state.should_quit {
                break;
            } else if self.state.should_run_command {
                self.run_command()?;
                self.state.should_run_command = false;
            }

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
