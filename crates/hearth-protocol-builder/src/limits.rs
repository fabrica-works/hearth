//! Each child machine retains its own configured bounds (render.md, 2.3).
use crate::{config, git_machine, records};
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub git: git_machine::limits::Limits,
    pub config: config::limits::Limits,
    pub printer: records::limits::Limits,
}
pub const SHIPPED: Limits =
    Limits { git: git_machine::limits::SHIPPED, config: config::limits::SHIPPED, printer: records::limits::SHIPPED };
