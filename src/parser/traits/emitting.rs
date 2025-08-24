pub trait WithInstructionEmitting {
    fn emit_instruction(&self, f: &mut String);

    fn into_emitted_instruction(&self) -> String {
        let mut out = String::new();
        self.emit_instruction(&mut out);

        out
    }
}
