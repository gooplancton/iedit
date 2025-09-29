use super::{command::EditorCommand, movement::MovementDirection, Editor};

pub enum EditorInput {
    CharInsertion(char),
    BasicMovement(MovementDirection),
    Command(EditorCommand),
}

impl Editor {
}
