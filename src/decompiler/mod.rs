pub mod ast;

mod traits;
pub use traits::WithCodeEmitting;
pub use traits::WithDecompiling;

mod iterator;
pub use iterator::DecompileNodeResult;
pub use iterator::DecompileResult;
pub use iterator::InstructionsIter;

mod emitter;
pub use emitter::CodeEmitter;

use prelude::*;

pub(crate) mod prelude {
    pub use super::DecompileNodeResult;
    pub use super::DecompileResult;
    pub use super::InstructionsIter;
    pub use super::WithDecompiling;

    pub use super::ast::*;
    pub(crate) use crate::parser;

    pub use parser::WithInstructionEmitting;
    pub use parser::ast::Instruction;

    pub use super::traits::WithCodeEmitting;
}
