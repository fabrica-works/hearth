//! Configured limits (render.md, 2.3 and 5; store.md, 6.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub path: u32,
    pub name: u32,
    pub request: u32,
    pub include: u32,
    pub line: u32,
    pub insts: u32,
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
    idle: skein_lib::Duration::from_secs(60),
};
