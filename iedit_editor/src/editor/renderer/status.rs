use std::io::Write;

use crate::{
    Editor,
    editor::{
        modes::EditorMode,
        renderer::{Renderer, line::LineRenderer},
    },
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
        if renderer.is_first_render {
            renderer.add_horizontal_bar()?;
        }
        renderer.next_line()?;
        renderer.clear_line()?;

        renderer.add("  ".as_bytes())?;

        if !self.status_bar.notification.is_empty() {
            return self.render_notification(renderer);
        }

        let content: &str = match self.mode {
            EditorMode::Insert => &format!(
                "{} | Ln: {}, Col: {}",
                self.get_displayable_file_path(),
                self.cursor.cur_y + 1,
                self.cursor.cur_x + 1
            ),
            EditorMode::Prompt(prompt) => {
                renderer.add(prompt)?;
                self.status_bar.prompt_line.as_ref()
            }
            EditorMode::Goto {
                original_cursor_pos: _,
            } => {
                renderer.add("GOTO ")?;
                self.status_bar.prompt_line.as_ref()
            }
            EditorMode::Search {
                original_cursor_pos: _,
                is_backwards,
            } => {
                if is_backwards {
                    renderer.add("BACK_")?;
                }
                renderer.add("SEARCH ")?;

                self.status_bar.prompt_line.as_ref()
            }
        };

        let mut line_renderer = LineRenderer::new(
            content,
            (0, self.ui.term_width as usize),
            &mut renderer.term,
            renderer.tab_size,
        );

        if !matches!(self.mode, EditorMode::Insert) {
            line_renderer.add_cursor(self.status_bar.cursor_pos);
        }

        line_renderer.render()?;

        Ok(())
    }
}
