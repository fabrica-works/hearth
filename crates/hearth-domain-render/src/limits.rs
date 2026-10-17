//! Configured render bounds (render.md, 2.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub targets: u32,
    pub errors: u32,
    pub tasks: u32,
    pub decoded: u64,
    pub validated: u64,
    pub document: u32,
    pub template: u32,
    pub statements: u32,
    pub iterations: u32,
    pub rendered_file: u32,
    pub rendered_target: u64,
    pub depth: u32,
    pub expression: u32,
    pub nesting: u32,
    pub made: u64,
    pub static_file: u32,
    pub hunk_file: u32,
    pub expansion: u32,
    pub expanded: u64,
    pub exported: u64,
}
pub const SHIPPED: Limits = Limits {
    targets: 1_024,
    errors: 256,
    tasks: 1_024,
    decoded: 67_108_864,
    validated: 67_108_864,
    document: 1_048_576,
    template: 1_048_576,
    statements: 32,
    iterations: 1_048_576,
    rendered_file: 16_777_216,
    rendered_target: 268_435_456,
    depth: 32,
    expression: 4_096,
    nesting: 64,
    made: 16_777_216,
    static_file: 67_108_864,
    hunk_file: 1_048_576,
    expansion: 65_536,
    expanded: 67_108_864,
    exported: 67_108_864,
};

/// A literal that is five bytes cannot fit, for budget refusal tests.
pub const FOUR_MADE: Limits = Limits { made: 4, ..SHIPPED };
