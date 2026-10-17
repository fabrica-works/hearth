//! Configured render bounds (render.md, 2.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub targets: u32,
    pub document: u32,
    pub depth: u32,
    pub expression: u32,
    pub nesting: u32,
    pub made: u64,
    pub static_file: u32,
}
pub const SHIPPED: Limits = Limits {
    targets: 1_024,
    document: 1_048_576,
    depth: 32,
    expression: 4_096,
    nesting: 64,
    made: 16_777_216,
    static_file: 67_108_864,
};

/// A literal that is five bytes cannot fit, for budget refusal tests.
pub const FOUR_MADE: Limits = Limits { made: 4, ..SHIPPED };
