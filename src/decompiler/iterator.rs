use std::slice::SliceIndex;

use crate::parser::ast::OperandValue;

use super::prelude::*;

#[derive(Copy, Clone)]
pub struct InstructionsIter<'a> {
    inner: &'a [Instruction],
    offset_limit: usize,
}

impl<'a> InstructionsIter<'a> {
    pub fn new(instructions: &'a Vec<Instruction>) -> Self {
        Self {
            inner: &instructions[..],
            offset_limit: 0,
        }
    }

    pub fn offset(&self) -> Option<usize> {
        self.peek().map(|i| i.offset)
    }

    pub fn offset_limit(&self) -> usize {
        self.offset_limit
    }

    pub fn next(mut self) -> (Self, Option<&'a Instruction>) {
        while let Some(instr) = self.inner.first() {
            self.inner = &self.inner[1..];

            if !self.instruction_is_within_offset_limit(instr) {
                return (self, None);
            }

            if !Self::instruction_should_be_skipped(instr) {
                return (self, Some(instr));
            }
        }

        (self, None)
    }

    pub fn next_raw(mut self) -> (Self, Option<&'a Instruction>) {
        while let Some(instr) = self.inner.first() {
            self.inner = &self.inner[1..];

            if !self.instruction_is_within_offset_limit(instr) {
                return (self, None);
            }

            if instr.mnemo == "Breakpoint" || !Self::instruction_should_be_skipped(instr) {
                return (self, Some(instr));
            }
        }

        (self, None)
    }

    pub fn peek(&self) -> Option<&'a Instruction> {
        for instr in self.inner {
            if !self.instruction_is_within_offset_limit(instr) {
                return None;
            }

            if !Self::instruction_should_be_skipped(instr) {
                return Some(instr);
            }
        }

        None
    }

    pub fn find_next(mut self, mnemo: &'static str) -> (Self, Option<&'a Instruction>) {
        while let Some(instr) = self.inner.first() {
            self.inner = &self.inner[1..];

            if !self.instruction_is_within_offset_limit(instr) {
                break;
            }

            if instr.mnemo == mnemo {
                return (self, Some(instr));
            } else {
                break;
            }
        }

        (self, None)
    }

    pub fn is_after_or_equal(&self, other: &Self) -> bool {
        // self.inner.len() <= other.inner.len()

        match (self.peek(), other.peek()) {
            (Some(a), Some(b)) => a.offset >= b.offset,
            // no more element
            (Some(a), None) => false,
            (None, Some(b)) => true,
            (None, None) => true,
        }
    }

    pub fn expect(mut self, mnemo: &'static str) -> DecompileResult<'a> {
        match self.next() {
            (i, Some(instr)) => {
                if instr.mnemo == mnemo {
                    Ok((i, instr))
                } else {
                    Err(format!("needed {mnemo} but found {}", instr.mnemo))
                }
            }
            _ => Err(format!("needed {mnemo} but found end of iterator")),
        }
    }

    pub fn expect_raw(mut self, mnemo: &'static str) -> DecompileResult<'a> {
        match self.next_raw() {
            (i, Some(instr)) => {
                if instr.mnemo == mnemo {
                    Ok((i, instr))
                } else {
                    Err(format!("needed {mnemo} but found {}", instr.mnemo))
                }
            }
            _ => Err(format!("needed {mnemo} but found end of iterator")),
        }
    }

    pub fn expect_any(self, mnemos: &[&'static str]) -> DecompileResult<'a> {
        match self.next() {
            (i, Some(instr)) => {
                if mnemos.contains(&instr.mnemo) {
                    Ok((i, instr))
                } else {
                    Err(format!("needed {mnemos:?} but found {}", instr.mnemo))
                }
            }
            _ => Err(format!("needed {mnemos:?} but found end of iterator")),
        }
    }

    pub fn maybe(mut self, mnemo: &'static str) -> (Self, Option<&'a Instruction>) {
        match self.next() {
            (i, Some(instr)) => {
                if instr.mnemo == mnemo {
                    (i, Some(instr))
                } else {
                    (i, None)
                }
            }
            (i, None) => (i, None),
        }
    }

    pub fn maybe_any(self, mnemos: &[&'static str]) -> (Self, Option<&'a Instruction>) {
        match self.next() {
            (i, Some(instr)) => {
                if mnemos.contains(&instr.mnemo) {
                    (i, Some(instr))
                } else {
                    (i, None)
                }
            }
            (i, None) => (i, None),
        }
    }

    /// utility function to convert a decompile result into an option
    pub fn ok<T>(&self, result: DecompileNodeResult<'a, T>) -> (Self, Option<T>) {
        match result {
            Ok((i, value)) => (i, Some(value)),
            Err(_) => (self.clone(), None),
        }
    }

    pub fn operand<'b>(
        &self,
        instruction: &'b Instruction,
        operand: &'static str,
    ) -> Result<&'b OperandValue, String> {
        use crate::parser::WithInstructionEmitting;

        match instruction.operands.get(operand) {
            Some(v) => Ok(v),
            None => Err(format!(
                "tried to get {operand} from {} but found None",
                instruction.into_emitted_instruction()
            )),
        }
    }

    pub fn within_offset_limit(mut self, instruction: &Instruction) -> Self {
        self.offset_limit = obtain_offset_limit(instruction).max(0) as usize;
        self
    }

    pub fn within_offset_limit_raw(mut self, instruction: &Instruction, raw_offset: usize) -> Self {
        self.offset_limit = instruction.offset + raw_offset;
        self
    }

    pub fn within_offset_limit_absolute(mut self, raw_offset: usize) -> Self {
        self.offset_limit = raw_offset;
        self
    }

    pub fn release_offset_limit(mut self) -> Self {
        self.offset_limit = 0;
        self
    }

    pub fn release_exhausted_offset_limit(mut self) -> Result<Self, String> {
        if self.offset_limit > 0
            && let Some(next) = self.peek()
        {
            if self.instruction_is_within_offset_limit(next) {
                return Err(format!(
                    "Expected Iterator to have exhausted offset at {}, but the following instruction was found: \n {next:?}",
                    self.offset_limit
                ));
            }
        }

        Ok(self.release_offset_limit())
    }

    pub fn is_finished(&self) -> bool {
        self.peek().is_none()
    }

    fn instruction_is_within_offset_limit(&self, instruction: &Instruction) -> bool {
        self.offset_limit <= 0 || instruction.offset <= self.offset_limit
    }

    /// Some instructions do not provide any valuable information for decompiling,
    /// these are skipped internally to keep the logic cleaner.
    fn instruction_should_be_skipped(instruction: &Instruction) -> bool {
        match instruction.mnemo {
            "Nop" | "Context" | "Breakpoint" | "Skip" => true,
            _ => false,
        }
    }
}

impl std::fmt::Debug for InstructionsIter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstructionsIter")
            .field("inner[0]", &self.inner.get(0))
            .field("offset_limit", &self.offset_limit)
            .finish()
    }
}

pub type DecompileNodeResult<'a, T> = Result<(InstructionsIter<'a>, T), String>;
pub type DecompileResult<'a> = DecompileNodeResult<'a, &'a Instruction>;

pub fn obtain_offset_limit(instruction: &parser::ast::Instruction) -> i64 {
    let base = (instruction.offset + instruction.size) as i64;

    match instruction
        .operands
        .get("skip_offset")
        .or_else(|| instruction.operands.get("unused")) // this one is used by switch labels
    {
        Some(parser::ast::OperandValue::Unsigned32(skip_offset)) => {
            base + *skip_offset as i64
        }
        Some(parser::ast::OperandValue::Unsigned16(skip_offset)) => {
            base + *skip_offset as i64
        }
        Some(parser::ast::OperandValue::Integer16(skip_offset)) => {
            base + *skip_offset as i64
        }
        Some(parser::ast::OperandValue::Integer32(skip_offset)) => {
            base + *skip_offset as i64
        }
        _ => base,
    }
}

#[test]
fn test_iterator_copy() {
    use parser::ast::Instruction;

    let instructions = vec![
        Instruction::new_fake("This", 0),
        Instruction::new_fake("NativeFunction", 1),
        Instruction::new_fake("Context", 2),
    ];

    let i = InstructionsIter::new(&instructions);
    let (i, _) = i.maybe("This");

    let (_, one) = i.clone().next();
    let (_, two) = i.clone().next();

    assert_eq!(two.map(|i| i.mnemo), one.map(|i| i.mnemo));
}

#[test]
fn test_iterator_filtering() {
    let instructions = vec![
        Instruction {
            mnemo: "NativeFunction",
            offset: 1,
            operands: std::collections::HashMap::from([(
                "skip_offset",
                parser::ast::OperandValue::Unsigned32(2),
            )]),
            opcode: 0,
            size: 1,
        },
        Instruction::new_fake("Context", 2), // should be skipped
        Instruction::new_fake("Nop", 3),     // should be skipped
        Instruction::new_fake("This", 4),
        Instruction::new_fake("Parent", 5),
    ];

    let base = InstructionsIter::new(&instructions);

    let (base, a) = base.maybe("NativeFunction");
    let (base, b) = base.maybe("This");

    assert_eq!(true, a.is_some() && b.is_some());
}

#[test]
fn test_iterator_within_limit() {
    use parser::ast::Instruction;

    let instructions = vec![
        Instruction {
            mnemo: "NativeFunction",
            offset: 1,
            operands: std::collections::HashMap::from([(
                "skip_offset",
                parser::ast::OperandValue::Unsigned32(2),
            )]),
            opcode: 0,
            size: 1,
        },
        Instruction::new_fake("This", 2),
        Instruction::new_fake("Context", 3),
        Instruction::new_fake("Nop", 4),
        Instruction::new_fake("Parent", 5),
    ];

    let mut base = InstructionsIter::new(&instructions);

    let mut iter = base.clone();
    if let Ok((iter, a)) = iter.expect("NativeFunction") {
        let limit = obtain_offset_limit(&a);
        assert_eq!(limit, 3);

        let (iter, a) = iter.within_offset_limit(a).maybe("This");
        // confirm that the iterator stopped before even reaching the Parent
        // that is after the offset limit
        let (iter, b) = iter.maybe("Parent");

        assert_eq!(true, a.is_some());
        assert_eq!(false, b.is_some());
    }
}

#[test]
fn test_search_progression() {
    use parser::ast::Instruction;

    let instructions = vec![
        Instruction::new_fake("This", 0),
        Instruction::new_fake("NativeFunction", 1),
        Instruction::new_fake("Parent", 2),
    ];

    let mut base = InstructionsIter::new(&instructions);

    // test a successful progression in the iter as all instructions are found
    let mut iter = base.clone();
    let (iter, a) = iter.maybe("This");
    let (iter, b) = iter.maybe("NativeFunction");
    let (iter, c) = iter.maybe("Parent");

    assert_eq!(true, a.is_some() && b.is_some() && c.is_some());

    // test a failed progression as the second search is invalid
    // in such case the iterator keeps progressing and the invalid item can be
    // ignored if the logic needs it.
    let mut iter = base.clone();
    let (iter, a) = iter.maybe("This");
    let (iter, b) = iter.maybe("IncorrectMatchInMiddle");
    let (iter, c) = iter.maybe("Parent");

    assert_eq!(true, a.is_some() && c.is_some());
    assert_eq!(false, b.is_some());

    // test a `any` search with a failed & successfull progression
    let mut iter = base.clone();
    let (iter, a) = iter.maybe("This");
    let (iter, b) = iter.maybe_any(&["IncorrectMatchInMiddle", "NativeFunction"]);
    let (iter, c) = iter.maybe("Parent");

    assert_eq!(true, a.is_some() && b.is_some() && c.is_some());
}
