/// Input buffer for the calculator expression input.
/// Handles cursor position, character insertion/deletion.
#[derive(Debug, Clone)]
pub struct InputBuffer {
    pub content: String,
    pub cursor_pos: usize,
}

impl InputBuffer {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor_pos: 0,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        let byte_pos = self.cursor_byte_pos();
        self.content.insert(byte_pos, ch);
        self.cursor_pos += 1;
    }

    pub fn insert_str(&mut self, s: &str) {
        let byte_pos = self.cursor_byte_pos();
        self.content.insert_str(byte_pos, s);
        self.cursor_pos += s.chars().count();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            let byte_pos = self.cursor_byte_pos();
            let before = self
                .content
                .char_indices()
                .nth(self.cursor_pos - 1)
                .map(|(i, c)| i + c.len_utf8())
                .unwrap_or(byte_pos);
            self.content.drain(before..byte_pos);
            self.cursor_pos -= 1;
        }
    }

    pub fn delete_forward(&mut self) {
        if self.cursor_pos < self.content.chars().count() {
            let byte_pos = self.cursor_byte_pos();
            let after = self
                .content
                .char_indices()
                .nth(self.cursor_pos)
                .map(|(i, c)| i + c.len_utf8())
                .unwrap_or(self.content.len());
            self.content.drain(byte_pos..after);
        }
    }

    pub fn cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor_pos < self.content.chars().count() {
            self.cursor_pos += 1;
        }
    }

    pub fn cursor_home(&mut self) {
        self.cursor_pos = 0;
    }

    pub fn cursor_end(&mut self) {
        self.cursor_pos = self.content.chars().count();
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor_pos = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn cursor_byte_pos(&self) -> usize {
        self.content
            .char_indices()
            .nth(self.cursor_pos)
            .map(|(i, _)| i)
            .unwrap_or(self.content.len())
    }
}

impl Default for InputBuffer {
    fn default() -> Self {
        Self::new()
    }
}
