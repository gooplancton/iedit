pub trait CharacterIndexable {
    fn n_chars(&self) -> usize;
    // fn char_to_byte_idx(&self, char_idx: usize) -> usize;
    fn byte_to_char_idx(&self, byte_idx: usize) -> Option<usize>;
    fn char_to_visual_idx(&self, char_idx: usize, tab_size: usize) -> usize;
    fn visual_to_char_idx(&self, char_idx: usize, tab_size: usize) -> usize;
    // fn split_chars_at(&self, char_idx: usize) -> (&Self, &Self);
    // fn split_chars_at_mut(&mut self, char_idx: usize) -> (&mut Self, &mut Self);
}

impl CharacterIndexable for str {
    #[inline]
    fn n_chars(&self) -> usize {
        self.chars().count()
    }

    // #[inline]
    // fn char_to_byte_idx(&self, char_idx: usize) -> usize {
    //     self.char_indices()
    //         .nth(char_idx)
    //         .map(|(i, _)| i)
    //         .unwrap_or(self.len())
    // }

    #[inline]
    fn byte_to_char_idx(&self, byte_idx: usize) -> Option<usize> {
        self.char_indices()
            .enumerate()
            .find(|(_, (char_byte_idx, _))| *char_byte_idx == byte_idx)
            .map(|(char_idx, _)| char_idx)
    }

    #[inline]
    fn char_to_visual_idx(&self, char_idx: usize, tab_size: usize) -> usize {
        self.chars()
            .take(char_idx)
            .map(|ch| if ch == '\t' { tab_size } else { 1 })
            .sum()
    }

    #[inline]
    fn visual_to_char_idx(&self, visual_idx: usize, tab_size: usize) -> usize {
        let mut running_visual_idx = 0;
        for (char_idx, ch) in self.chars().enumerate() {
            running_visual_idx += if ch == '\t' { tab_size } else { 1 };
            if running_visual_idx > visual_idx {
                return char_idx;
            }
        }

        self.len()
    }

    // #[inline]
    // fn split_chars_at(&self, char_idx: usize) -> (&Self, &Self) {
    //     let byte_idx = self.char_to_byte_idx(char_idx);
    //     self.split_at(byte_idx)
    // }

    // fn split_chars_at_mut(&mut self, char_idx: usize) -> (&mut Self, &mut Self) {
    //     let byte_idx = self.char_to_byte_idx(char_idx);
    //     self.split_at_mut(byte_idx)
    // }
}
