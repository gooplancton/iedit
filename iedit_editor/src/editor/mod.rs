#![allow(unused)]

use std::{
    fmt::Display,
    fs::File,
    io::{BufWriter, Stdout, Write, stdin, stdout},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use config::EditorConfig;
use iedit_document::Document;
use signal_hook::{consts::SIGWINCH, flag};
use termion::{
    cursor::{DetectCursorPos, Goto, HideCursor},
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
    screen::IntoAlternateScreen,
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

use crossbeam_channel::unbounded;

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

// Store sender in a static or global location for access anywhere
lazy_static::lazy_static! {
    pub static ref NOTIFICATION_SENDER: std::sync::Mutex<Option<crossbeam_channel::Sender<String>>> =
        std::sync::Mutex::new(None);
}

impl Editor {
    pub fn new(path: impl AsRef<Path>, open_at_line: usize) -> std::io::Result<Self> {
        let (file, canonicalized_file_path, file_lines) = read_file(path)?;

        let config = if let Some(mut path) = std::env::home_dir() {
            path.push(".iedit.conf");
            EditorConfig::from_file(path).unwrap_or_default()
        } else {
            EditorConfig::default()
        };

        let open_at_line = file.as_ref().map(|_| open_at_line).unwrap_or_default();
        let cur_y = open_at_line.saturating_sub(1);

        let document = Document::new(file_lines);
        let renderer = Renderer::new(config.n_lines)?;
        let viewport = Viewport::new(renderer.editor_lines, open_at_line);

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

    pub fn reset_ui(&mut self) {
        // need to figure something out here
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        self.render()?;

        let window_resized = Arc::<AtomicBool>::new(AtomicBool::new(false));
        flag::register(SIGWINCH, window_resized.clone());

        let (notification_sender, notification_receiver) = unbounded();

        *NOTIFICATION_SENDER.lock().unwrap() = Some(notification_sender);

        let input_parser = InputParser::new(notification_receiver);
        for input in input_parser {
            self.cursor.set_last_pos();

            if window_resized
                .compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                self.reset_ui();
            }

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
