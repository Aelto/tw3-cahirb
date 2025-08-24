use crate::parser::prelude::*;

#[derive(Debug)]
pub struct InternalOperatorRef {
    pub table_index: usize,
}

impl WithTableResolving<&'static str> for InternalOperatorRef {
    fn try_resolve<'a>(&self) -> Option<&&'static str> {
        TableManager::try_resolve_internal_operator(self.table_index)
    }
}

impl WithInstructionEmitting for InternalOperatorRef {
    fn emit_instruction(&self, f: &mut String) {
        use std::fmt::Write;

        match self.try_resolve() {
            Some(operator) => write!(f, "{operator}").unwrap(),
            None => {
                write!(f, "__unresolved_internal_operator__").unwrap();
            }
        }
    }
}
