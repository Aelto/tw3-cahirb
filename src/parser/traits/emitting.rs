pub trait WithCodeEmitting {
    fn emit_code(&self, f: &mut String);

    fn into_emitted_code(&self) -> String {
        let mut out = String::new();
        self.emit_code(&mut out);

        out
    }
}
