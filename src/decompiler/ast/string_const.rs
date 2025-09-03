use crate::decompiler::prelude::*;

#[derive(Debug)]
pub struct StringConst {
    name: String,
}

impl WithDecompiling for StringConst {
    fn decompile<'a>(i: InstructionsIter<'a>) -> DecompileNodeResult<'a, Self> {
        let (i, name) = i.expect("StringConst")?;

        Ok((
            i,
            Self {
                name: name.to_string(),
            },
        ))
    }
}

impl WithCodeEmitting for StringConst {
    fn emit_code(&self, f: &mut crate::decompiler::CodeEmitter) {
        f.append("\"");
        f.append(&self.name);
        f.append("\"");
    }
}
