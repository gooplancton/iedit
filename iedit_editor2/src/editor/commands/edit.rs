use iedit_document::EditOperation;

use crate::Editor;

impl Editor {
    pub fn apply_edit_side_effects(&mut self, op: &EditOperation) {
        match op {
            EditOperation::InsertCharAt { pos, ch } => {
                let (ins_x, ins_y) = *pos;
                self.cursor.update_pos((ins_x + 1, ins_y));
            }
            EditOperation::DeleteCharAt { pos } => {
                let (del_x, del_y) = *pos;
                self.cursor.update_pos((del_x.saturating_sub(1), del_y));
            }
            EditOperation::InsertStringAt { pos, string } => {
                let (ins_x, ins_y) = *pos;
                self.cursor.update_pos((ins_x + string.len(), ins_y));
            }
            EditOperation::DeleteRange { x_from, x_to, y } => {
                self.cursor.update_pos((*x_from, *y));
            }
            EditOperation::ReplaceRange {
                x_from,
                x_to,
                y,
                string,
            } => todo!(),
            EditOperation::InsertStringsAtMultiline { pos, strings } => {
                let (_, ins_y) = *pos;
                self.cursor.update_pos((0, ins_y + strings.len()));
            }
            EditOperation::ReplaceRangeMultiline {
                pos_from,
                pos_to,
                strings,
            } => {
                let (ins_x, ins_y) = *pos_from;
                let landing_x = strings
                    .last()
                    .map(|l| l.chars().count())
                    .unwrap_or_default();
                self.cursor.update_pos((landing_x, ins_y + strings.len()));
            }
            EditOperation::DeleteRangeMultiline { pos_from, pos_to } => {
                self.cursor.update_pos(*pos_from);
            }
        }
    }
}
