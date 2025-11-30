use crate::Document;

#[derive(Default)]
pub struct DocumentVocabulary {
    /// character in the trie node, \0 represents the root
    ch: char,
    /// full word ending with char
    word: Option<String>,
    /// higher for more frequent words
    rank: usize,
    children: Vec<DocumentVocabulary>,
}

impl DocumentVocabulary {
    pub fn register_word(&mut self, word: &str) {
        let mut root = self;
        for ch in word.chars() {
            let next_idx = root.get_child_idx(ch).unwrap_or_else(|| {
                root.children.push(Self {
                    ch,
                    ..Default::default()
                });
                root.children.len() - 1
            });

            let next = &mut root.children[next_idx];
            next.rank += 1;
            root = next;
        }

        root.word = Some(word.to_string());
    }

    pub fn get_child_idx(&self, ch: char) -> Option<usize> {
        self.children.iter().position(|child| child.ch == ch)
    }

    pub fn get_words_with_prefix(&self, prefix: &str) -> Vec<&str> {
        let mut root = self;

        for ch in prefix.chars() {
            let next_idx = root.get_child_idx(ch);
            if next_idx.is_none() {
                return vec![];
            }

            let next = &root.children[next_idx.unwrap()];
            root = next;
        }

        let mut words = vec![];
        let mut visitable: Vec<&DocumentVocabulary> = vec![root];
        while let Some(node) = visitable.pop() {
            if let Some(word) = &node.word {
                words.push(word.as_ref());
            }
            node.children.iter().for_each(|child| visitable.push(child));
        }

        words
    }
}

impl Document {
    pub fn init_vocabulary(&mut self) {
        let vocab = &mut self.vocabulary;
        for line in &mut self.lines {
            line.as_ref()
                .split(|ch: char| ch != '_' && !ch.is_alphabetic())
                .filter(|word| word.len() >= 3)
                .for_each(|word| vocab.register_word(word));
        }
    }
}
