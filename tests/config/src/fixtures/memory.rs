//! Small documents retain the same array-growth and schema-conversion peaks.
use hearth_protocol_config::limits::{Limits, SHIPPED};
// A 4-KiB document keeps these allocator cases cheap; width eight and
// depth four include list growth, map finishing and a depth refusal.
pub const JSON: Limits = Limits { document: SHIPPED.document / 256, depth: 4, keys: 8, settings: 8 };
// Schema frames need the sixteen nested levels of the type fixture.
pub const DECLARATION: Limits = Limits { depth: 16, ..JSON };
