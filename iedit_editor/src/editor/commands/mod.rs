mod execute;
mod move_cursor;
mod notify;

use crate::{editor::modes::EditorMode};
use iedit_document::EditOperation;
pub use move_cursor::MoveCursor;
use termion::event::Key;

pub use notify::send_notification;
pub enum EditorCommand {
    Quit,
    Save,
    MoveCursor {
        movement: MoveCursor,
        with_selection: bool,
    },
    ToggleLockSelection,
    ClearSelection,
    SwitchMode(EditorMode),
    Edit(EditOperation),
    UndoLastEdit,
    RedoLastEdit,
    MovePromptCursorLeft,
    MovePromptCursorRight,
    InsertCharPrompt {
        pos_x: usize,
        ch: char,
    },
    DeleteCharPrompt {
        pos_x: usize,
    },
    SubmitPrompt,
    ToggleLineNumbers,
    ScrollViewportUp,
    ScrollViewportDown,
    FindMatchForward,
    FindMatchBackward,
    ExecuteFile(Key),
    DisplayExternalNotification(String),
}

#[non_exhaustive]
pub enum CommandExecutionResult {
    Continue,
    ShouldQuit,
}
