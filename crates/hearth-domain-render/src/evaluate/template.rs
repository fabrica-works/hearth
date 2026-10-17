//! Flat template instructions (format.md, 7.2; render.md, 8.3).
use super::Expression;
use alloc::boxed::Box;
/// A compiled template, with the first byte of each line.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Template {
    pub instructions: Box<[Instruction]>,
    pub lines: Box<[u32]>,
}
/// One instruction at its opening tag or first text byte.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Instruction {
    pub at: u32,
    pub does: Does,
}
/// The work an instruction performs.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Does {
    Text(Box<[u8]>),
    Output(Expression),
    Branch { condition: Expression, over: u32 },
    Jump { over: u32 },
    Enter { names: Names, iterable: Expression, over: u32 },
    Advance { back: u32 },
    Set { name: Box<[u8]>, value: Expression },
}

/// Names bound by one loop (format.md, 7.3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Names {
    One { name: Box<[u8]> },
    Pair { first: Box<[u8]>, second: Box<[u8]> },
}

use super::{Bindings, Budget, Context, Evaluator, Fault, FaultKind, Written, expression, frame, output, truthy};
use crate::{limits::Limits, revision::Position};
use skein_lib::List;
/// Runs a flat program, bounded by the total instructions and iterations.
pub fn run<'c>(
    template: &'c Template,
    context: &Context<'c>,
    element: Option<(super::Operand<'c>, super::Operand<'c>)>,
    limits: &Limits,
    room: u64,
) -> Result<Box<[u8]>, Fault> {
    let mut runner = Runner {
        evaluator: Evaluator::new(limits),
        bindings: Bindings::with_capacity(
            limits.template.saturating_div(11).saturating_add(limits.statements.saturating_mul(3)).saturating_add(2),
        ),
        frames: skein_lib::Stack::with_capacity(limits.statements),
        budget: Budget::new(limits),
        iterations: limits.iterations,
        bytes: List::with_capacity(4096.min(limits.rendered_file)),
        room: room.min(u64::from(limits.rendered_file)),
    };
    if let Some((key, value)) = element {
        located(frame::push(&mut runner.bindings, b"key", key), 0)?;
        located(frame::push(&mut runner.bindings, b"value", value), 0)?;
    }
    let mut pc = 0_usize;
    let steps = template
        .instructions
        .len()
        .saturating_mul(usize::try_from(limits.iterations).expect("limit fits").saturating_add(1));
    for _ in 0..=steps {
        let Some(instruction) = template.instructions.get(pc) else { return Ok(runner.bytes.into_boxed()) };
        pc = runner.step(instruction, context, pc)?;
    }
    unreachable!("instructions times iterations bounds a compiled program")
}
#[derive(Debug)]
struct Runner<'c> {
    evaluator: Evaluator<'c>,
    bindings: Bindings<'c>,
    frames: skein_lib::Stack<frame::Frame<'c>>,
    budget: Budget,
    iterations: u32,
    bytes: List<u8>,
    room: u64,
}
impl<'c> Runner<'c> {
    fn step(&mut self, instruction: &'c Instruction, context: &Context<'c>, pc: usize) -> Result<usize, Fault> {
        let at = instruction.at;
        let mut next = pc.saturating_add(1);
        let result = match &instruction.does {
            Does::Text(bytes) => self.write(bytes),
            Does::Output(code) => {
                let value = expression(&mut self.evaluator, code, context, &self.bindings, &mut { self.budget })?;
                match output(&value) {
                    Ok(Written::Bytes(bytes)) => self.write(bytes),
                    Ok(Written::Integer { negative, digits }) => {
                        if negative {
                            located(self.write(b"-"), at)?;
                        }
                        self.write(digits.as_bytes())
                    }
                    Err(kind) => Err(kind),
                }
            }
            Does::Branch { condition, over } => {
                let value = expression(&mut self.evaluator, condition, context, &self.bindings, &mut { self.budget })?;
                if !truthy(&value) {
                    next = next.saturating_add(usize::try_from(*over).expect("jump fits"));
                }
                Ok(())
            }
            Does::Jump { over } => {
                next = next.saturating_add(usize::try_from(*over).expect("jump fits"));
                Ok(())
            }
            Does::Set { name, value } => {
                let value = expression(&mut self.evaluator, value, context, &self.bindings, &mut self.budget)?;
                match value {
                    super::Operand::Undefined => Err(FaultKind::Undefined),
                    super::Operand::Loop { .. } => Err(FaultKind::Type),
                    super::Operand::Bool(_)
                    | super::Operand::Int(_)
                    | super::Operand::String(_)
                    | super::Operand::List { .. }
                    | super::Operand::Value(_) => frame::push(&mut self.bindings, name, value),
                }
            }
            Does::Enter { names, iterable, over } => {
                let outer = self.budget;
                let value = expression(&mut self.evaluator, iterable, context, &self.bindings, &mut self.budget)?;
                let mut frame =
                    located(frame::Frame::new(value, names, at, self.bindings.len(), outer, self.budget), at)?;
                if located(frame.bind(&mut self.bindings, &mut self.budget, &mut self.iterations), at)? {
                    match self.frames.push(frame) {
                        Ok(()) => Ok(()),
                        Err(_frame) => Err(FaultKind::Limit),
                    }
                } else {
                    self.budget = outer;
                    next = next.saturating_add(usize::try_from(*over).expect("jump fits"));
                    Ok(())
                }
            }
            Does::Advance { back } => {
                let frame = self.frames.top_mut().expect("compiled loop");
                self.bindings.truncate(frame.base);
                self.budget = frame.iteration;
                if located(frame.bind(&mut self.bindings, &mut self.budget, &mut self.iterations), frame.at)? {
                    next = pc.checked_sub(usize::try_from(*back).expect("jump fits")).expect("compiled back jump");
                } else {
                    self.budget = frame.outer;
                    drop(self.frames.pop());
                }
                Ok(())
            }
        };
        located(result, at)?;
        Ok(next)
    }
    fn write(&mut self, bytes: &[u8]) -> Result<(), FaultKind> {
        let length = u64::from(self.bytes.len()).saturating_add(u64::try_from(bytes.len()).expect("bytes fit"));
        if length > self.room {
            return Err(FaultKind::Limit);
        }
        if length > u64::from(self.bytes.capacity()) {
            let mut capacity = self.bytes.capacity().max(1);
            for _ in 0..u32::BITS {
                if u64::from(capacity) >= length {
                    break;
                }
                capacity = capacity.saturating_mul(2).min(u32::try_from(self.room).expect("file bound"));
            }
            let mut grown = List::with_capacity(capacity);
            for byte in &self.bytes {
                grown.push(*byte).expect("grown capacity");
            }
            self.bytes = grown;
        }
        for byte in bytes {
            self.bytes.push(*byte).expect("measured length");
        }
        Ok(())
    }
}
/// Locates an instruction's byte in the compiler's line table.
#[must_use]
pub fn place(lines: &[u32], at: u32) -> Position {
    let index = match lines.binary_search(&at) {
        Ok(index) => index,
        Err(index) => index.saturating_sub(1),
    };
    Position {
        line: u32::try_from(index).expect("bounded lines").saturating_add(1),
        column: at.saturating_sub(*lines.get(index).expect("first line is zero")).saturating_add(1),
    }
}

fn located<T>(result: Result<T, FaultKind>, at: u32) -> Result<T, Fault> {
    match result {
        Ok(value) => Ok(value),
        Err(kind) => Err(Fault { at, kind }),
    }
}

/// The most heap held by one run, including regrowth of its output.
/// Returns `None` when the bound does not fit a `u64`.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    let bindings = limits.template.checked_div(11)?.checked_add(limits.statements.checked_mul(3)?)?.checked_add(2)?;
    let output = u64::from(limits.rendered_file);
    // The preceding doubling allocation is half the next power of two.
    // This is 1.5 times the shipped file cap, including its replacement.
    let regrowth = output.checked_add(output.saturating_sub(1).checked_next_power_of_two()?.checked_div(2)?)?;
    super::worst_case(limits)?
        .checked_add(skein_lib::Stack::<frame::Frame<'_>>::worst_case(limits.statements)?)?
        .checked_add(List::<super::Binding<'_>>::worst_case(bindings)?)?
        .checked_add(regrowth)?
        .checked_add(limits.made)
}
