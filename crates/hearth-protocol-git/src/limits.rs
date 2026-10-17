//! Configured bounds for git's stream machines (render.md, 2.3).
use alloc::boxed::Box;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub tail_lines: u32,
    pub tail_line: u32,
    pub entries: u32,
    pub path: u32,
    pub paths_total: u32,
    pub chunk: u32,
    pub hunk_lines: u32,
    pub hunk_file: u32,
}
pub const SHIPPED: Limits = Limits {
    tail_lines: 20,
    tail_line: 160,
    entries: 65_536,
    path: 4_096,
    paths_total: 16_777_216,
    chunk: 16_384,
    hunk_lines: 200,
    hunk_file: 1_048_576,
};

/// The decoder whose retained heap is being priced.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Decoder {
    Tail,
    Listing,
    Batch,
    Diff,
}
/// Decoder-owned heap, including one delivery; emitted payloads are the receiver's.
#[must_use]
pub fn worst_case(decoder: Decoder, limits: &Limits) -> Option<u64> {
    match decoder {
        Decoder::Tail => skein_lib::Queue::<Box<[u8]>>::worst_case(limits.tail_lines)?
            .checked_add(u64::from(limits.tail_lines).checked_mul(u64::from(limits.tail_line))?)?
            .checked_add(u64::from(limits.tail_line).checked_mul(2)?),
        Decoder::Listing => skein_lib::List::<crate::listing::Entry>::worst_case(limits.entries)?
            .checked_add(u64::from(limits.paths_total))?
            .checked_add(u64::from(crate::listing::RECORD).checked_add(u64::from(limits.path))?),
        // Two files of retained text, a reusable line buffer, and one delivery.
        Decoder::Diff => skein_lib::List::<crate::hunks::Line>::worst_case(limits.hunk_lines)?
            .checked_add(u64::from(limits.hunk_file).checked_mul(3)?)?
            .checked_add(u64::from(limits.chunk)),
        Decoder::Batch => Some(u64::from(crate::batch::largest_demand(limits))),
    }
}
