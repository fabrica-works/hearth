/// Configured expression bounds (render.md, 2.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub expression: u32,
    pub nesting: u32,
}
/// The shipped profile (render.md, 2.3).
pub const SHIPPED: Limits = Limits { expression: 4096, nesting: 64 };
