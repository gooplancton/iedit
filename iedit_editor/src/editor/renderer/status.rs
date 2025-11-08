use std::io::Write;

use iedit_document::CharacterIndexable;

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

        if matches!(self.mode, EditorMode::Insert) {
            let mut status_text_len = 0;
            for chunk in self.get_status_text_chunks() {
                status_text_len += chunk.n_chars();
                renderer.add(chunk)?;
            }

            let cursor_pos_chunk = format!(
                "{}:{} - {}%",
                self.cursor.cur_y + 1,
                self.cursor.cur_x + 1,
                (100 * self.cursor.cur_y / self.document.n_lines()).min(100)
            );

            let padding = (self.ui.term_width as usize)
                .saturating_sub(4)
                .saturating_sub(status_text_len)
                .saturating_sub(cursor_pos_chunk.len());
            renderer.add(" ".repeat(padding))?;
            renderer.add(cursor_pos_chunk)?;

            return Ok(());
        }

        let content = match self.mode {
            EditorMode::Insert => unreachable!(),
            EditorMode::Prompt(prompt) => {
                renderer.add(prompt)?;
                &self.status_bar.prompt_line
            }
            EditorMode::Goto {
                original_cursor_pos: _,
            } => {
                renderer.add("GOTO ")?;
                &self.status_bar.prompt_line
            }
            EditorMode::Search {
                original_cursor_pos: _,
                is_backwards,
            } => {
                if is_backwards {
                    renderer.add("BACK_")?;
                }
                renderer.add("SEARCH ")?;

                &self.status_bar.prompt_line
            }
        };

        let mut line_renderer = LineRenderer::new(
            content,
            0,
            self.ui.term_width as usize,
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
