//! Configured limits (domain/activator.md, 2.3; domain/store.md, 5.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Limits {
    /// The most bytes of one file.
    pub file: u32,
    /// The most lines of one file.
    pub lines: u32,
    /// The most bytes of one name.
    pub name: u32,
}
/// Design defaults: the shipped profile.
pub const SHIPPED: Limits = Limits { file: 1_048_576, lines: 4_096, name: 256 };
