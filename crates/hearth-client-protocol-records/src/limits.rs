//! Printer admission limits (render.md, 10).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// The most bytes one record prints, including line endings.
    pub record: u32,
}
/// The shipped profile: 64 MiB per record.
pub const SHIPPED: Limits = Limits { record: 67_108_864 };
