//! Printer admission limits (render.md, 10).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// The most bytes one record prints, including line endings.
    pub record: u32,
    /// The most room demanded at once, at least one.
    pub chunk: u32,
}
/// The shipped profile: 16 MiB per record.
pub const SHIPPED: Limits = Limits { record: 16_777_216, chunk: 65_536 };
