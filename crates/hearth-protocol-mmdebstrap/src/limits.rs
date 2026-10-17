//! Configured limits (render.md, 2.3 and 5; store.md, 6.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub path: u32,
    pub name: u32,
    pub request: u32,
    pub include: u32,
    pub line: u32,
    pub insts: u32,
    pub tail_lines: u32,
    pub tail_line: u32,
    pub stanza: u32,
    pub chunk: u32,
    pub filename: u32,
    pub idle: skein_lib::Duration,
}
/// Design defaults: the shipped profile.
pub const SHIPPED: Limits = Limits {
    path: 4_096,
    name: 256,
    request: 8_192,
    include: 65_536,
    line: 4_096,
    insts: 16_384,
    tail_lines: 20,
    tail_line: 160,
    stanza: 1_048_576,
    chunk: 16_384,
    filename: 4_096,
    idle: skein_lib::Duration::from_secs(60),
};
