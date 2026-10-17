/// Configured expression bounds (render.md, 2.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub expression: u32,
    pub nesting: u32,
    pub template: u32,
}
/// The shipped profile (render.md, 2.3).
pub const SHIPPED: Limits = Limits { expression: 4096, nesting: 64, template: 1_048_576 };

/// One of the crate's entry points.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Decoder {
    /// `expression::compile`.
    Expression,
}
/// The most heap a decoder holds at once, its returned expression included;
/// `None` when the bound does not fit a `u64`.
#[must_use]
pub fn worst_case(decoder: Decoder, limits: &Limits) -> Option<u64> {
    match decoder {
        Decoder::Expression => crate::expression::worst_case(limits.expression),
    }
}
