//! Configured render bounds (render.md, 2.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub targets: u32,
    pub document: u32,
}
pub const SHIPPED: Limits = Limits { targets: 1_024, document: 1_048_576 };
