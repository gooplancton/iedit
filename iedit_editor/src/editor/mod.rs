#![allow(unused)]

use std::{
    fmt::Display,
    fs::File,
    io::{BufWriter, Stdout, Write, stdin},
    path::{Path, PathBuf},
};

use config::EditorConfig;
use iedit_document::Document;
use termion::{
    cursor::{DetectCursorPos, Goto, HideCursor},
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
    terminal_size,
};

use crate::{
    editor::{
        commands::CommandExecutionResult, cursor::Cursor, io::read_file, modes::EditorMode,
        renderer::Renderer, status::StatusBar, viewport::Viewport,
    },
    input::InputParser,
    terminal::H_BAR,
};

mod commands;
mod config;
mod cursor;
mod highlight;
mod io;
mod modes;
mod renderer;
mod status;
mod viewport;

pub struct Editor {
    config: EditorConfig,
    file: Option<File>,
    document: Document,
    canonicalized_file_path: PathBuf,
    mode: EditorMode,
    status_bar: StatusBar,
    cursor: Cursor,
    viewport: Viewport,
    renderer: Renderer,

    // could be a bitfield?
    is_selection_locked: bool,
    first_quit_sent: bool,
    is_executing_file: bool,
}

impl Editor {
    pub fn new(path: impl AsRef<Path>, open_at: usize) -> std::io::Result<Self> {
        let (file, canonicalized_file_path, file_lines) = read_file(path)?;

        let (term_width, term_height) = terminal_size()?;
        let mut term = HideCursor::from(std::io::stdout().into_raw_mode()?);
        let mut ui_origin = term.cursor_pos()?;

        let mut config = if let Some(mut path) = std::env::home_dir() {
            path.push(".iedit.conf");
            EditorConfig::from_file(path).unwrap_or_default()
        } else {
            EditorConfig::default()
        };

        let document = Document::new(file_lines);

        let real_estate = term_height.saturating_sub(ui_origin.1);
        if config.n_lines == 0 || real_estate < config.n_lines {
            let offset = config.set_default_n_lines(&mut term, ui_origin.1)?;
            ui_origin.1 = ui_origin.1.saturating_sub(offset);
        }

        let cur_y = open_at.saturating_sub(1);

        let mut viewport = Viewport::default();
        viewport.pre_scroll_top_line = open_at.saturating_sub(config.n_lines as usize / 2);
        viewport.top_line = viewport.pre_scroll_top_line;

        let horizontal_bar = str::repeat(H_BAR, term_width as usize);
        let renderer = Renderer {
            term: BufWriter::new(term),
            ui_origin,
            term_width,
            horizontal_bar,
            dirty_lines: vec![],
            needs_full_rerender: true,
        };

        Ok(Self {
            file,
            canonicalized_file_path,
            document,
            mode: EditorMode::Insert,
            config,
            status_bar: StatusBar::default(),
            cursor: Cursor::new((0, cur_y)),
            renderer,
            viewport,
            is_selection_locked: false,
            first_quit_sent: false,
            is_executing_file: false,
        })
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        self.render()?;

        let stdin = stdin();
        let input_parser = InputParser { keys: stdin.keys() };

        for input in input_parser {
            self.cursor.set_last_pos();

            let command = self.parse_command(input);
            if command.is_none() {
                continue;
            }

            let command = command.unwrap();
            let res = self.execute_command(command);

            if matches!(res, CommandExecutionResult::ShouldQuit) {
                break;
            }

            self.clamp_cursor();

            self.adjust_viewport();

            self.render()?;
        }

        self.cleanup()?;
        Ok(())
    }

    pub fn get_displayable_file_path(&self) -> impl Display {
        if self.canonicalized_file_path.as_os_str().is_empty() {
            "[Unnamed Buffer]".to_owned()
        } else {
            format!(
                "{}{}",
                self.canonicalized_file_path.as_os_str().to_string_lossy(),
                if self.document.has_been_edited {
                    "*"
                } else {
                    ""
                }
            )
        }
    }
}
