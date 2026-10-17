//! Configured bounds for git's stream machines (render.md, 2.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub entries: u32,
    pub path: u32,
    pub paths_total: u32,
    pub chunk: u32,
}
pub const SHIPPED: Limits = Limits { entries: 65_536, path: 4_096, paths_total: 16_777_216, chunk: 16_384 };
