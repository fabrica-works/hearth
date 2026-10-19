/// Shipped listing entries (domain/render.md, 2.3).
pub const LISTING_ENTRIES: u64 = 16_384;
/// Shipped document bytes (domain/render.md, 2.3).
pub const DOCUMENT_BYTES: u64 = 262_144;
/// A listed path may hold 4 KiB (domain/render.md, 2.3).
pub const PATH_BYTES: usize = 4_096;
/// One listed path is enough to cross the per-path allowance.
pub const SINGLE_ENTRY: u32 = 1;
/// Two thirty-byte paths fit in this smaller total; the third fails.
pub const PATHS_BYTES: u32 = 64;
pub const LISTING_PATH: &[u8] = b"a-path-that-is-thirty-bytes-long";
pub const PATH_COPIES: u32 = 3;
