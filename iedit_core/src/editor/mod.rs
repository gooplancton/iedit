use std::{
    fs::File,
    io::{Stdout, Write, stdin},
    path::{Path, PathBuf},
};

use config::EditorConfig;
use state::EditorState;
use termion::{
    cursor::{DetectCursorPos, Goto, HideCursor},
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
    terminal_size,
};

use crate::{
    editor::{input::process_key_event, io::read_file},
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
    ui_origin: (u16, u16),
    term_width: u16,

    horizontal_bar: String,
    temp_message: String,

    dirty_lines: Vec<usize>,
    needs_full_rerender: bool,
}

impl<TextLine: EditorLine> Editor<TextLine> {
    pub fn new(path: impl AsRef<Path>, open_at: usize) -> std::io::Result<Self> {
        let (file, canonicalized_file_path, file_lines) = read_file(path)?;
        let mut state = EditorState::default();

        let (term_width, term_height) = terminal_size()?;
        let mut term = HideCursor::from(std::io::stdout().into_raw_mode()?);
        let mut ui_origin = term.cursor_pos()?;

        let mut config = if let Some(mut path) = std::env::home_dir() {
            path.push(".iedit.conf");
            EditorConfig::from_file(path).unwrap_or_default()
        } else {
            EditorConfig::default()
        };

        let real_estate = term_height.saturating_sub(ui_origin.1);
        if config.n_lines == 0 || real_estate < config.n_lines {
            let offset = config.set_default_n_lines(&mut term, ui_origin.1)?;
            ui_origin.1 = ui_origin.1.saturating_sub(offset);
        }

        state.cursor_pos_y = open_at.saturating_sub(1);
        state.viewport.pre_scroll_top_line = open_at.saturating_sub(config.n_lines as usize / 2);
        state.viewport.top_line = state.viewport.pre_scroll_top_line;
        state.viewport.bottom_line = state.viewport.top_line + config.n_lines as usize;
        state.viewport.right_col = (term_width as usize) - (config.show_line_numbers as usize * 7);

        let horizontal_bar = str::repeat(H_BAR, term_width as usize);
        let temp_message = String::with_capacity(10);

        Ok(Self {
            file,
            canonicalized_file_path,
            file_lines,
            state,
            config,
            term,
            term_width,
            ui_origin,
            horizontal_bar,
            temp_message,
            dirty_lines: vec![],
            needs_full_rerender: true,
        })
    }

    pub fn quit(&mut self) -> std::io::Result<()> {
        self.state.should_quit = true;
        Ok(())
    }

    pub fn reset_cursor(&mut self) -> std::io::Result<()> {
        write!(
            &mut self.term,
            "{}",
            Goto(self.ui_origin.0, self.ui_origin.1)
        )
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        self.render()?;

        let stdin = stdin();
        for key in stdin.keys() {
            let key = key?;

            let prev_x = self.state.cursor_pos_x as isize;
            let prev_y = self.state.cursor_pos_y as isize;

            self.state.cursor_previous_pos_y = self.state.cursor_pos_y;

            let input = process_key_event(key);

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
