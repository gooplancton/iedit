use std::{
    fmt::Display,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::config::EditorConfig;
use iedit_document::Document;
use signal_hook::{consts::SIGWINCH, flag};

use crate::{
    editor::{
        commands::CommandExecutionResult, cursor::Cursor, io::read_file, modes::EditorMode,
        renderer::Renderer, status::StatusBar, viewport::Viewport,
    },
    input::InputParser,
    terminal::UILayout,
};

use crossbeam_channel::{Sender, unbounded};

mod commands;
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
    ui: UILayout,
    dirty_lines: Vec<usize>,

    // could be a bitfield?
    needs_full_rerender: bool,
    is_selection_locked: bool,
    is_readonly: bool,
    first_quit_sent: bool,
    is_executing_file: bool,
}

// Store sender in a static or global location for access anywhere
pub static NOTIFICATION_SENDER: Mutex<Option<Sender<String>>> = Mutex::new(None);
pub static FILE_EXECUTION_OUTPUT: Mutex<Option<Document>> = Mutex::new(None);

impl Editor {
    pub fn new(
        file_path: impl AsRef<Path>,
        open_at_line: usize,
        config: EditorConfig,
        ui: UILayout,
    ) -> std::io::Result<Self> {
        let (file, canonicalized_file_path, file_lines) = read_file(file_path)?;

        let open_at_line = file.as_ref().map(|_| open_at_line).unwrap_or_default();
        let cur_y = open_at_line.saturating_sub(1);

        let document = Document::new(file_lines, Some(config.tab_size));
        let viewport = Viewport::new(ui.editor_lines, open_at_line);

        Ok(Self {
            file,
            canonicalized_file_path,
            document,
            mode: EditorMode::Insert,
            config,
            status_bar: StatusBar::default(),
            cursor: Cursor::new((0, cur_y)),
            ui,
            viewport,
            dirty_lines: vec![],
            needs_full_rerender: true,
            is_readonly: false,
            is_selection_locked: false,
            first_quit_sent: false,
            is_executing_file: false,
        })
    }

    pub fn toggle_execution_output(&mut self) {
        if let Ok(mut execution_output) = FILE_EXECUTION_OUTPUT.lock() {
            if let Some(execution_output) = execution_output.as_mut() {
                self.is_readonly = !self.is_readonly;
                self.swap_docuemnt(execution_output);
            }
        }
    }

    pub fn swap_docuemnt(&mut self, new_doc: &mut Document) {
        std::mem::swap(&mut self.document, new_doc);
        self.needs_full_rerender = true;
    }

    pub fn reset_ui(&mut self) {
        // need to figure something out here
    }

    pub fn run<Term: Write>(&mut self, term: &mut Term) -> std::io::Result<()> {
        let mut renderer = Renderer::new(term, self.ui.clone());
        renderer.render(self)?;

        let window_resized = Arc::<AtomicBool>::new(AtomicBool::new(false));
        let _ = flag::register(SIGWINCH, window_resized.clone());

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

            renderer.render(self)?;

            self.dirty_lines.truncate(0);
            self.status_bar.notification.truncate(0);
            // TODO: set to false once we figure out dirty_lines
            self.needs_full_rerender = true;
        }

        renderer.cleanup()?;

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
