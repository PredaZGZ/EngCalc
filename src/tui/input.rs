/// Input buffer for the calculator expression input.
/// Stores content as a Vec<char> for correct cursor positioning.
#[derive(Debug, Clone)]
pub struct InputBuffer {
    chars: Vec<char>,
    cursor_pos: usize,
}

impl InputBuffer {
    pub fn new() -> Self {
        Self {
            chars: Vec::new(),
            cursor_pos: 0,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.chars.insert(self.cursor_pos, ch);
        self.cursor_pos += 1;
    }

    #[allow(dead_code)]
    pub fn insert_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.chars.insert(self.cursor_pos, ch);
            self.cursor_pos += 1;
        }
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            self.chars.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
        }
    }

    pub fn delete_forward(&mut self) {
        if self.cursor_pos < self.chars.len() {
            self.chars.remove(self.cursor_pos);
        }
    }

    pub fn cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor_pos < self.chars.len() {
            self.cursor_pos += 1;
        }
    }

    pub fn cursor_home(&mut self) {
        self.cursor_pos = 0;
    }

    pub fn cursor_end(&mut self) {
        self.cursor_pos = self.chars.len();
    }

    pub fn clear(&mut self) {
        self.chars.clear();
        self.cursor_pos = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    pub fn content(&self) -> String {
        self.chars.iter().collect()
    }

    pub fn cursor_pos(&self) -> usize {
        self.cursor_pos
    }

    pub fn set_content(&mut self, s: String) {
        self.chars = s.chars().collect();
        self.cursor_pos = self.chars.len();
    }

    pub fn set_cursor_pos(&mut self, pos: usize) {
        self.cursor_pos = pos.min(self.chars.len());
    }
}

impl Default for InputBuffer {
    fn default() -> Self {
        Self::new()
    }
}
