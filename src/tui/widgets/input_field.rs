pub struct InputField<'a> {
    buffer: &'a str,
    is_password: bool,
}

impl<'a> InputField<'a> {
    pub fn new(buffer: &'a str) -> Self {
        Self {
            buffer,
            is_password: false,
        }
    }

    pub fn with_password(mut self, val: bool) -> Self {
        self.is_password = val;
        self
    }

    pub fn render(&self) -> String {
        if self.is_password {
            "*".repeat(self.buffer.len())
        } else {
            self.buffer.to_string()
        }
    }
}
