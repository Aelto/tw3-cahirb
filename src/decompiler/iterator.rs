use std::slice::SliceIndex;

use super::prelude::*;

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

    pub fn ok<T>(&self, result: DecompileNodeResult<'a, T>) -> (Self, Option<T>) {
        match result {
            Ok((i, value)) => (i, Some(value)),
            Err(_) => (self.clone(), None),
        }
    }

    pub fn within_offset_limit(mut self, limit: usize) -> Self {
        self.offset_limit = limit;
        self
    }

    fn instruction_is_within_offset_limit(&self, instruction: &Instruction) -> bool {
        self.offset_limit <= 0 || instruction.offset <= self.offset_limit
    }

    /// Some instructions do not provide any valuable information for decompiling,
    /// these are skipped internally to keep the logic cleaner.
    fn instruction_should_be_skipped(instruction: &Instruction) -> bool {
        match instruction.mnemo {
            "Nop" | "Context" | "Breakpoint" => true,
            _ => false,
        }
    }
}

impl<'a> Clone for InstructionsIter<'a> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner,
            offset_limit: self.offset_limit.clone(),
        }
    }
}

pub type DecompileNodeResult<'a, T> = Result<(InstructionsIter<'a>, T), String>;
pub type DecompileResult<'a> = DecompileNodeResult<'a, &'a Instruction>;

pub fn obtain_offset_limit(instruction: &parser::ast::Instruction) -> usize {
    match instruction.operands.get("skip_offset") {
        Some(parser::ast::OperandValue::Unsigned32(skip_offset)) => {
            instruction.offset + *skip_offset as usize
        }
        Some(parser::ast::OperandValue::Unsigned16(skip_offset)) => {
            instruction.offset + *skip_offset as usize
        }
        _ => instruction.offset + instruction.size,
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

        let (iter, a) = iter.within_offset_limit(limit).maybe("This");
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
