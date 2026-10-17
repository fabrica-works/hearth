//! Each child machine retains its own configured bounds (render.md, 2.3).
use crate::{config, git_machine, records};
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub mmdebstrap: hearth_protocol_mmdebstrap::limits::Limits,
    pub walk: Walk,
    pub git: git_machine::limits::Limits,
    pub config: config::limits::Limits,
    pub template: hearth_protocol_template::limits::Limits,
    pub tree: crate::tree::limits::Limits,
    pub printer: records::limits::Limits,
}
/// The bounded staging walk (store.md, 4.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Walk {
    pub depth: u32,
    pub page: u32,
    pub entries: u32,
}
pub const SHIPPED: Limits = Limits {
    mmdebstrap: hearth_protocol_mmdebstrap::limits::SHIPPED,
    walk: Walk { depth: 16, page: 64, entries: 65_536 },
    tree: crate::tree::limits::SHIPPED,
    git: git_machine::limits::SHIPPED,
    config: config::limits::SHIPPED,
    template: hearth_protocol_template::limits::SHIPPED,
    printer: records::limits::SHIPPED,
};
