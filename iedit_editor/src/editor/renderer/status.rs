use std::{cmp::max, io::Write};

use crate::{
    Editor,
    editor::{
        modes::EditorMode,
        renderer::{Renderer, line::LineRenderer},
        search::SearchItem,
        status::KEYBINDINGS,
    },
    terminal::CLEAR_TO_END_OF_LINE,
};

impl Editor {
    pub fn render_notification<'renderer, 'term, Term: Write>(
        &self,
        renderer: &'renderer mut Renderer<'term, Term>,
    ) -> std::io::Result<()> {
        renderer.add(&self.status_bar.notification)?;
        renderer.add(CLEAR_TO_END_OF_LINE)?;
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
        renderer.add("  ".as_bytes())?;

        if !self.status_bar.notification.is_empty() {
            return self.render_notification(renderer);
        }

        if matches!(self.mode, EditorMode::Insert) {
            let mut left_status_len = 0;
            let document_lines = max(self.document.n_lines(), 1);
            for flag_str in self.get_flag_strings() {
                left_status_len += flag_str.len() - 13;
                renderer.add(flag_str)?;
            }

            let cursor_pos_chunk = format!(
                "   {}:{} - {}%",
                self.cursor.cur_y + 1,
                self.cursor.cur_x + 1,
                (100 * self.cursor.cur_y / document_lines).min(100)
            );

            left_status_len += cursor_pos_chunk.len();
            renderer.add(cursor_pos_chunk)?;

            if self.config.show_keybindings {
                let padding = (self.ui.term_width as usize)
                    .saturating_sub(left_status_len)
                    .saturating_sub(KEYBINDINGS.len());
                renderer.add(" ".repeat(padding))?;
                renderer.add(&KEYBINDINGS)?;
            }

            renderer.add(CLEAR_TO_END_OF_LINE)?;

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

                match self.search_item {
                    Some(SearchItem::PromptString) => {
                        renderer.add("(exact) ")?;
                    }
                    _ => {
                        renderer.add("(regex) ")?;
                    }
                }
                &self.status_bar.prompt_line
            }
        };

        let mut line_renderer = LineRenderer::new(
            content,
            0,
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
