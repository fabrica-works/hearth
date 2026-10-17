//! The record limits shared by this world.
use hearth_protocol_tree::limits::{Limits, SHIPPED};

#[must_use]
pub const fn limits() -> Limits {
    SHIPPED
}
