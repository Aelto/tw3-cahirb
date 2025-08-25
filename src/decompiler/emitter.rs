pub struct CodeEmitter {
    buffer: String,

    indent_level: usize,
}

impl CodeEmitter {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            indent_level: 0,
        }
    }

    pub fn buffer(&mut self) -> &mut String {
        &mut self.buffer
    }

    pub fn append(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    pub fn linebreak(&mut self) {
        self.buffer.push('\r');
    }

    pub fn indent(&mut self) {
        for _ in 0..self.indent_level {
            self.buffer.push('\t');
        }
    }

    pub fn add_indent(&mut self) {
        self.indent_level += 1;
    }

    pub fn remove_indent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    pub fn to_string(&self) -> &str {
        &self.buffer
    }
}
