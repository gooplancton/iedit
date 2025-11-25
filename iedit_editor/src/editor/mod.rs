use std::{
    cmp::min,
    io::Write,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{
    config::EditorConfig,
    editor::{
        clipboard::{EditorClipboard, get_clipboard},
        search::SearchItem,
    },
    input::Notification,
};
use iedit_document::Document;
use signal_hook::{consts::SIGWINCH, flag};

use crate::{
    editor::{
        commands::CommandExecutionResult, cursor::Cursor, modes::EditorMode, renderer::Renderer,
        status::StatusBar, viewport::Viewport,
    },
    input::InputParser,
    terminal::UILayout,
};

use crossbeam_channel::{Sender, unbounded};

mod clipboard;
mod commands;
mod cursor;
mod highlight;
mod io;
mod keybindings;
mod modes;
mod renderer;
mod search;
mod status;
mod viewport;

pub struct Editor {
    config: EditorConfig,
    document: Document,
    mode: EditorMode,
    status_bar: StatusBar,
    cursor: Cursor,
    viewport: Viewport,
    ui: UILayout,
    search_item: Option<SearchItem>,
    matched_range: Option<((usize, usize), (usize, usize))>,
    displayed_popup: Option<&'static [&'static str]>,
    clipboard: Box<dyn EditorClipboard>,

    // TODO: turn into EditorFlags bitfield
    needs_full_rerender: bool,
    is_selection_locked: bool,
    first_quit_sent: bool,
    is_running_external_command: bool,
    is_viewing_execution_output: bool,
}

// Store sender in a static or global location for access anywhere
pub static NOTIFICATION_SENDER: Mutex<Option<Sender<Notification>>> = Mutex::new(None);
pub static FILE_EXECUTION_OUTPUT: Mutex<Option<Document>> = Mutex::new(None);

pub enum EditorRunResult {
    RestartInFullscreenMode,
    Quit,
}

impl Editor {
    pub fn new(
        document: Document,
        open_at_line: usize,
        config: EditorConfig,
        ui: UILayout,
    ) -> std::io::Result<Self> {
        let viewport = Viewport::new(ui.editor_lines, open_at_line);
        let clipboard = get_clipboard(config.use_system_clipboard);

        let cur_y = min(open_at_line.saturating_sub(1), document.n_lines());

        Ok(Self {
            document,
            mode: EditorMode::Insert,
            config,
            status_bar: StatusBar::default(),
            cursor: Cursor::new((0, cur_y)),
            ui,
            viewport,
            clipboard,
            search_item: None,
            matched_range: None,
            displayed_popup: None,
            needs_full_rerender: true,
            is_selection_locked: false,
            first_quit_sent: false,
            is_running_external_command: false,
            is_viewing_execution_output: false,
        })
    }

    pub fn toggle_execution_output(&mut self) {
        if let Ok(mut execution_output) = FILE_EXECUTION_OUTPUT.lock()
            && let Some(execution_output) = execution_output.as_mut()
        {
            self.is_viewing_execution_output = !self.is_viewing_execution_output;
            self.swap_docuemnt(execution_output);
        }
    }

    pub fn set_ui(&mut self, ui: UILayout) {
        self.ui = ui;
    }

    pub fn swap_docuemnt(&mut self, new_doc: &mut Document) {
        std::mem::swap(&mut self.document, new_doc);
        self.needs_full_rerender = true;
    }

    pub fn run<Term: Write>(&mut self, term: &mut Term) -> std::io::Result<EditorRunResult> {
        let mut renderer = Renderer::new(term, self.ui.clone(), self.config.tab_size as usize);
        renderer.render(self)?;

        let window_resized = Arc::<AtomicBool>::new(AtomicBool::new(false));
        let _ = flag::register(SIGWINCH, window_resized.clone());

        let (notification_sender, notification_receiver) = unbounded::<Notification>();

        *NOTIFICATION_SENDER.lock().unwrap() = Some(notification_sender);

        let input_parser = InputParser::new(notification_receiver);
        for input in input_parser {
            self.cursor.set_last_pos();
            let gutter_width_before = self.get_line_number_gutter_width();

            if window_resized
                .compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return Ok(EditorRunResult::RestartInFullscreenMode);
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

            let gutter_width_after = self.get_line_number_gutter_width();
            self.needs_full_rerender |= gutter_width_after != gutter_width_before;

            self.adjust_viewport();

            renderer.render(self)?;

            self.document.reset_lines_need_render(
                self.viewport.top_line..self.viewport.top_line + self.ui.editor_lines as usize,
            );
            self.status_bar.notification.truncate(0);
            self.needs_full_rerender = self.displayed_popup.is_some();
            self.displayed_popup = None;
        }

        renderer.cleanup()?;

        return Ok(EditorRunResult::Quit);
    }
}
