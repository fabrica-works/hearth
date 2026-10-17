use crate::{Filter, Op};
/// Explicit parser nesting, bounded by `Limits::nesting` (render.md, 2.3).
#[derive(Debug)]
pub(super) enum Pending {
    Not { at: u32 },
    Binary { op: Op, at: u32 },
    Group { start: u32 },
    List { start: u32, length: u32, at: u32 },
    Index { start: u32, at: u32, chain: (u32, bool) },
    Arguments { start: u32, filter: Filter, given: u32, at: u32 },
}
