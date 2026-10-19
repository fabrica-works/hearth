use crate::{Filter, Op};
/// Explicit parser nesting, bounded by `Limits::nesting` (domain/render.md, 2.3).
#[derive(Debug)]
pub(super) enum Pending {
    And { jump: u32 },
    Or { jump: u32 },
    If { start: u32, mid: u32, at: u32 },
    Else { jump: u32, start: u32 },
    Not { at: u32 },
    Binary { op: Op, at: u32 },
    Group { start: u32 },
    List { start: u32, mark: u32, length: u32, at: u32 },
    Index { start: u32, at: u32, chain: (u32, bool) },
    Arguments { start: u32, mark: u32, filter: Filter, given: u32, at: u32 },
}
