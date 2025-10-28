use crate::{
    Editor,
    editor::{
        highlight::SelectionHighlight, modes::EditorMode,
        renderer::legacy_line_renderer::LineRenderer,
    },
    terminal::{CLEAR_LINE, CURSOR_DOWN1, CURSOR_TO_COL1, EMPTY_CURSOR},
};

impl Editor {
    fn render_notification(&mut self) -> std::io::Result<()> {
        self.renderer.add(&self.status_bar.notification)?;

        self.renderer.add(CURSOR_DOWN1.as_bytes())?;
        self.renderer.add(CURSOR_TO_COL1.as_bytes())?;

        self.renderer.add(CLEAR_LINE.as_bytes())?;

        self.status_bar.notification.truncate(0);

        Ok(())
    }

    pub fn render_status(&mut self) -> std::io::Result<()> {
        self.renderer.add(CLEAR_LINE.as_bytes())?;
        self.renderer.add_horizontal_bar()?;
        self.renderer.add(CURSOR_DOWN1.as_bytes())?;
        self.renderer.add(CURSOR_TO_COL1.as_bytes())?;

        self.renderer.add(CLEAR_LINE.as_bytes())?;
        self.renderer.add("  ".as_bytes())?;

        if !self.status_bar.notification.is_empty() {
            return self.render_notification();
        }

        let content: &String = match self.mode {
            EditorMode::Insert => &format!(
                "{} | Ln: {}, Col: {}",
                self.get_displayable_file_path(),
                self.cursor.cur_y,
                self.cursor.cur_x
            ),
            EditorMode::Prompt(prompt) => {
                self.renderer.add(prompt);
                &self.status_bar.prompt_line
            }
            EditorMode::Goto => {
                self.renderer.add("GOTO ");
                &self.status_bar.prompt_line
            }
            EditorMode::Search => todo!(),
        };

        let mut renderer =
            LineRenderer::new(content).with_display_range(0..self.renderer.term_width as usize);

        if !matches!(self.mode, EditorMode::Insert) {
            let x = self.status_bar.cursor_pos;
            let selection_highlight = SelectionHighlight::Range(x, x + 1);
            renderer = renderer.with_selection_highlight(selection_highlight);
        }

        renderer.render_to(&mut self.renderer.term)?;

        let is_cursor_at_end_of_line =
            !matches!(self.mode, EditorMode::Insert) && self.status_bar.cursor_pos == content.len();
        if is_cursor_at_end_of_line {
            self.renderer.add(EMPTY_CURSOR.as_bytes())?;
        }

        self.renderer.add(CURSOR_DOWN1.as_bytes())?;
        self.renderer.add(CURSOR_TO_COL1.as_bytes())?;

        self.renderer.add(CLEAR_LINE.as_bytes())?;

        Ok(())
    }
}
