mod execute;
mod move_cursor;
mod notify;

use crate::editor::modes::EditorMode;
pub use execute::Executor;
use iedit_document::EditOperation;
pub use move_cursor::CursorMovement;
use termion::event::Key;

pub use notify::{send_simple_notification};
pub enum EditorCommand {
    Quit,
    Save,
    MoveCursor {
        movement: CursorMovement,
        with_selection: bool,
    },
    ToggleLockSelection,
    ClearSelection,
    SwitchMode(EditorMode),
    Edit(EditOperation),
    YankSelection,
    CutSelection,
    Paste,
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
    PromptExecutor,
    ExecuteFile(Key),
    DisplayMessage(String),
    DisplayHelp,
    DisplayChordsHelp,
    DisplayExecuteChordHelp,
    DisplayLineChordHelp,
    DisplayViewChordHelp,
    DisplaySelectionChordHelp,
    DisplayPressCharacterPopup,
    ViewExecutionOutput,
}

#[non_exhaustive]
pub enum CommandExecutionResult {
    Continue,
    ShouldQuit,
}
