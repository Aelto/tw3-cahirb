use std::slice::SliceIndex;

use super::prelude::*;

pub struct InstructionsIter<'a> {
    inner: &'a [Instruction],
    index: usize,
}

impl<'a> InstructionsIter<'a> {
    pub fn new(instruction: &'a Vec<Instruction>) -> Self {
        Self {
            inner: &instruction[..],
            index: 0,
        }
    }

    pub fn next(&mut self) -> Option<&Instruction> {
        while let Some(instr) = self.inner.first() {
            self.inner = &self.inner[1..];

            if !Self::instruction_should_be_skipped(instr) {
                return Some(instr);
            }
        }

        None
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
            inner: self.inner.clone(),
            index: self.index.clone(),
        }
    }
}

/// Trait aliasing for what's essentially an iterator over instructions, the
/// key type used in all functions below
pub trait InstructionsIterator<'a>: Iterator<Item = &'a Instruction> + Clone {}
impl<'a, T> InstructionsIterator<'a> for T where T: Iterator<Item = &'a Instruction> + Clone {}

pub type DecompileResult<'a> = Result<&'a parser::ast::Instruction, String>;
pub type DecompileNodeResult<'a, ITER: InstructionsIterator<'a>, T> = Result<(ITER, T), String>;

pub fn instructions_iter<'a>(
    instructions: &'a Vec<Instruction>,
) -> impl Iterator<Item = &'a Instruction> + Clone {
    instructions.iter().filter(|instr| match instr.mnemo {
        "Nop" | "Context" | "Breakpoint" => false,
        _ => true,
    })
}

pub fn expect<'a>(
    i: &mut impl Iterator<Item = &'a parser::ast::Instruction>,
    mnemo: &'static str,
) -> DecompileResult<'a> {
    match i.next() {
        Some(instr) => {
            if instr.mnemo == mnemo {
                Ok(instr)
            } else {
                Err(format!("needed {mnemo} but found {}", instr.mnemo))
            }
        }
        None => Err(format!("needed {mnemo} but found end of iterator")),
    }
}

pub fn expect_any<'a>(
    i: &mut impl Iterator<Item = &'a parser::ast::Instruction>,
    mnemos: &[&'static str],
) -> DecompileResult<'a> {
    match i.next() {
        Some(instr) => {
            if mnemos.contains(&instr.mnemo) {
                Ok(instr)
            } else {
                Err(format!("needed {mnemos:?} but found {}", instr.mnemo))
            }
        }
        None => Err(format!("needed {mnemos:?} but found end of iterator")),
    }
}

pub fn maybe<'a, T, ITER, ITER2>(
    decompile_result: DecompileNodeResult<'a, ITER, T>,
    base_iter: ITER,
) -> (ITER, Option<T>)
where
    ITER: InstructionsIterator<'a>,
{
    match decompile_result {
        Ok((i, node)) => (i, Some(node)),
        Err(_) => (base_iter, None),
    }
}

pub fn within_offset_limit<'a>(
    i: impl Iterator<Item = &'a parser::ast::Instruction>,
    limit: usize,
) -> impl Iterator<Item = &'a parser::ast::Instruction> {
    i.take_while(move |instr| instr.offset <= limit)
}

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

    let mut base = instructions.iter().peekable().skip(1);
    let one = base.next().map(|i| i.mnemo);
    let two = base.next().map(|i| i.mnemo);

    assert_eq!(two, one);
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

    let mut base = instructions.iter().peekable();

    let mut iter = base.clone();
    if let Ok(a) = expect(&mut iter, "NativeFunction") {
        let limit = obtain_offset_limit(&a);
        assert_eq!(limit, 3);

        let mut iter = within_offset_limit(iter, limit);

        let a = expect(&mut iter, "This");
        let b = expect(&mut iter, "Context");
        let c = expect(&mut iter, "Nop"); // the iterator stopped before this one
        let d = expect(&mut iter, "Parent");

        assert_eq!(true, a.is_ok() && b.is_ok());
        assert_eq!(false, c.is_ok() || d.is_ok());
    }
}

#[test]
fn test_search_progression() {
    use parser::ast::Instruction;

    let instructions = vec![
        Instruction::new_fake("This", 0),
        Instruction::new_fake("NativeFunction", 1),
        Instruction::new_fake("Context", 2),
    ];

    let mut base = instructions.iter().peekable();

    // test a successful progression in the iter as all instructions are found
    let mut iter = base.clone();
    let a = expect(&mut iter, "This");
    let b = expect(&mut iter, "NativeFunction");
    let c = expect(&mut iter, "Context");

    assert_eq!(true, a.is_ok() && b.is_ok() && c.is_ok());

    // test a failed progression as the second search is invalid
    // in such case the iterator keeps progressing and the invalid item can be
    // ignored if the logic needs it.
    let mut iter = base.clone();
    let a = expect(&mut iter, "This");
    let b = expect(&mut iter, "IncorrectMatchInMiddle");
    let c = expect(&mut iter, "Context");

    assert_eq!(true, a.is_ok() && c.is_ok());
    assert_eq!(false, b.is_ok());

    // test a `any` search with a failed & successfull progression
    let mut iter = base.clone();
    let a = expect(&mut iter, "This");
    let b = expect_any(&mut iter, &["IncorrectMatchInMiddle", "NativeFunction"]);
    let c = expect(&mut iter, "Context");

    assert_eq!(true, a.is_ok() && b.is_ok() && c.is_ok());
}
