use crate::{Instruction, Operation};
use skein_lib::{List, Stack};
/// Configured expression bounds (render.md, 2.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub expression: u32,
    pub nesting: u32,
    pub template: u32,
    pub statements: u32,
}
/// The shipped profile (render.md, 2.3).
pub const SHIPPED: Limits = Limits { expression: 4096, nesting: 64, template: 1_048_576, statements: 32 };

/// One of the crate's entry points.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Decoder {
    /// `expression::compile`.
    Expression,
    /// `template::compile`.
    Template,
    /// `interpolation::compile`, over decoded text bounded by its caller.
    Interpolated { length: u32 },
}
/// The most heap a decoder holds at once, its returned instructions or expression included;
/// `None` when the bound does not fit a `u64`.
#[must_use]
pub fn worst_case(decoder: Decoder, limits: &Limits) -> Option<u64> {
    match decoder {
        Decoder::Expression => crate::expression::worst_case(limits.expression),
        Decoder::Interpolated { length } => compiled(length, limits),
        Decoder::Template => compiled(limits.template, limits)?
            .checked_add(List::<u32>::worst_case(limits.template.checked_add(1)?)?.checked_mul(2)?)?
            .checked_add(Stack::<crate::template::statement::Block>::worst_case(
                limits.statements.min(limits.template),
            )?)?
            .checked_add(List::<u32>::worst_case(limits.template)?),
    }
}

// The scanner counts openings before allocating; each can emit three instructions.
pub(crate) fn instruction_capacity(openings: u32) -> u32 {
    openings.saturating_mul(3).saturating_add(1)
}
fn compiled(length: u32, limits: &Limits) -> Option<u64> {
    // Shrinking to boxed instructions can briefly hold both allocations.
    List::<Instruction>::worst_case(instruction_capacity(length))?
        .checked_mul(2)?
        .checked_add(List::<Operation>::worst_case(length)?)?
        // Retained text, strings and names are disjoint slices of the input.
        .checked_add(u64::from(length))?
        // One expression is compiled while previous instructions remain held.
        .checked_add(crate::expression::worst_case(limits.expression)?)
}
