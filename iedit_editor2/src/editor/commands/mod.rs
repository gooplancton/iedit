mod move_cursor;

use crate::{Editor, editor::modes::EditorMode};
use iedit_document::Edit;
pub use move_cursor::MoveCursor;
pub enum EditorCommand {
    Quit,
    Save,
    MoveCursor {
        movement: MoveCursor,
        with_selection: bool,
    },
    ClearSelection,
    SwitchMode(EditorMode),
    Edit {
        op: Edit,
        clear_selection: bool
    },
    UndoLastEdit,
    RedoLastEdit,
    MovePromptCursorLeft,
    MovePromptCursorRight,
    InsertCharPrompt {
        pos_x: usize,
        ch: char
    },
    DeleteCharPrompt {
        pos_x: usize
    },
    SubmitPrompt,
    ToggleLineNumbers,
    ScrollViewportUp,
    ScrollViewportDown,
    FindMatchForward,
    FindMatchBackward,
}

#[non_exhaustive]
pub enum CommandExecutionResult {
    Continue,
    ShouldQuit,
}
