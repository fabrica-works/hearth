use hearth_protocol_git::limits::{self, Limits};
pub const SHIPPED: Limits = limits::SHIPPED;
/// Small enough to exercise every limit without a shipped-size allocation.
pub const SMALL: Limits = Limits { entries: 8, path: 32, paths_total: 256, chunk: 16 };
