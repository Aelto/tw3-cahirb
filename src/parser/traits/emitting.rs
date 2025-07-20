pub trait WithCodeEmitting {
    fn emit_code(&self, f: &mut String);
}
