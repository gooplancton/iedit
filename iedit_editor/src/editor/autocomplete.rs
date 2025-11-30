use std::cmp::min;

use crate::Editor;

#[derive(Default)]
pub struct EditorAutocomplete {
    pub choices: Vec<String>, // TODO: figure out a better way
    pub selected_idx: usize,
    pub written_offset: usize,
}

impl EditorAutocomplete {
    pub fn get_displayable_choices(&self, max_choices: usize) -> &[String] {
        let start = min(
            (self.selected_idx + 1).saturating_sub(max_choices),
            self.choices.len().saturating_sub(max_choices),
        );
        let end = min(start + max_choices, self.choices.len());

        self.choices.as_slice().get(start..end).unwrap_or_default()
    }

    #[inline(always)]
    pub fn get_selected_choice(&mut self) -> String {
        if self.selected_idx >= self.choices.len() {
            return String::default();
        }

        let mut full_word = self.choices.swap_remove(self.selected_idx);
        full_word.split_off(self.written_offset)
    }
}

impl Editor {
    pub fn update_autocomplete_choices(&mut self, prefix_start_idx: usize) {
        self.autocomplete.selected_idx = 0;
        self.autocomplete.written_offset = self.cursor.cur_x - prefix_start_idx;

        let prefix =
            self.document.lines[self.cursor.cur_y].get_range(prefix_start_idx..self.cursor.cur_x);

        let autocomplete_choices = self.document.vocabulary.get_words_with_prefix(prefix);

        self.autocomplete.choices = autocomplete_choices
            .iter()
            .map(|choice| choice.to_string())
            .collect();
    }
}
