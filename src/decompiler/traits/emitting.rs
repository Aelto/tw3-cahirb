use crate::decompiler::CodeEmitter;

pub trait WithCodeEmitting {
    fn emit_code(&self, f: &mut CodeEmitter);
}
