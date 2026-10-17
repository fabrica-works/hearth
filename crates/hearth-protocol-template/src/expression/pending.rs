use crate::Filter;
/// Explicit parser nesting, bounded by `Limits::nesting` (render.md, 2.3).
#[derive(Debug)]
pub(super) enum Pending {
    Group { start: u32 },
    List { start: u32, length: u32, at: u32 },
    Index { start: u32, at: u32 },
    Arguments { start: u32, filter: Filter, given: u32, at: u32 },
}
