use std::io::Write;

use crate::{
    Editor,
    editor::{
        highlight::SelectionHighlight,
        modes::EditorMode,
        renderer::{Renderer, legacy_line_renderer::LineRenderer},
    },
    terminal::EMPTY_CURSOR,
};

impl Editor {
    pub fn render_notification<'renderer, 'term, Term: Write>(
        &self,
        renderer: &'renderer mut Renderer<'term, Term>,
    ) -> std::io::Result<()> {
        renderer.add(&self.status_bar.notification)?;
        renderer.next_line()?;

        Ok(())
    }

    pub fn render_status<'renderer, 'term, Term: Write>(
        &self,
        renderer: &'renderer mut Renderer<'term, Term>,
    ) -> std::io::Result<()> {
        renderer.add_horizontal_bar()?;
        renderer.next_line()?;

        renderer.add("  ".as_bytes())?;

        if !self.status_bar.notification.is_empty() {
            return self.render_notification(renderer);
        }

        let content: &String = match self.mode {
            EditorMode::Insert => &format!(
                "{} | Ln: {}, Col: {}",
                self.get_displayable_file_path(),
                self.cursor.cur_y + 1,
                self.cursor.cur_x + 1
            ),
            EditorMode::Prompt(prompt) => {
                renderer.add(prompt)?;
                &self.status_bar.prompt_line
            }
            EditorMode::Goto(_) => {
                renderer.add("GOTO ")?;
                &self.status_bar.prompt_line
            }
            EditorMode::Search => todo!(),
        };

        let mut line_renderer = LineRenderer::new(content, renderer.tab_size)
            .with_display_range(0..self.ui.term_width as usize);

        if !matches!(self.mode, EditorMode::Insert) {
            let x = self.status_bar.cursor_pos;
            let selection_highlight = SelectionHighlight::Range(x, x + 1);
            line_renderer = line_renderer.with_selection_highlight(selection_highlight);
        }

        line_renderer.render_to(&mut renderer.term)?;

        let is_cursor_at_end_of_line =
            !matches!(self.mode, EditorMode::Insert) && self.status_bar.cursor_pos == content.len();
        if is_cursor_at_end_of_line {
            renderer.add(EMPTY_CURSOR.as_bytes())?;
        }

        renderer.next_line()?;

        Ok(())
    }
}
