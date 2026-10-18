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

/// Each child machine once, with returned records and forwarding payloads
/// retained by this layer (programming-model.md, 6.3).
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    use alloc::boxed::Box;
    use git_machine::limits::{Decoder, worst_case as git};
    use hearth_protocol_template::limits::{Decoder as Template, worst_case as template};
    use skein_lib::{List, Queue};
    let child = git(Decoder::Tail, &limits.git)?.checked_mul(2)?.checked_add(
        git(Decoder::Listing, &limits.git)?
            .max(git(Decoder::Batch, &limits.git)?)
            .max(git(Decoder::Diff, &limits.git)?),
    )?;
    let decode = config::declaration::worst_case(&limits.config)?.max(template(Template::Template, &limits.template)?);
    let records = crate::tree::limits::worst_case(crate::tree::frame::Kind::Resolution, &limits.tree)?
        .checked_add(crate::tree::limits::held(crate::tree::frame::Kind::Resolution, &limits.tree)?)?;
    let resolve = hearth_protocol_mmdebstrap::output::worst_case(&limits.mmdebstrap)?
        .checked_mul(2)?
        .checked_add(hearth_protocol_mmdebstrap::packages::worst_case(&limits.mmdebstrap)?)?
        .checked_add(u64::from(limits.mmdebstrap.include).checked_mul(4)?)?
        .checked_add(u64::from(limits.mmdebstrap.request).checked_mul(u64::from(limits.mmdebstrap.name))?)?
        .checked_add(u64::from(limits.mmdebstrap.line).checked_mul(u64::from(limits.tree.resolution.lines))?)?;
    let walk = List::<skein_io::file::Entry>::worst_case(limits.walk.page)?
        .checked_add(u64::from(limits.walk.page).checked_mul(255)?)?
        .checked_add(255)?
        .checked_mul(u64::from(limits.walk.depth))?;
    let listing = List::<crate::render::vocabulary::Blob>::worst_case(limits.git.entries)?
        .checked_add(u64::from(limits.git.paths_total))?
        .checked_add(u64::from(limits.git.entries).checked_mul(64)?)?;
    let queues = Queue::<crate::Asked>::worst_case(4)?.checked_add(Queue::<Box<[u8]>>::worst_case(4)?)?;
    child
        .checked_add(decode)?
        .checked_add(records.checked_mul(4)?)?
        .checked_add(resolve)?
        .checked_add(walk)?
        .checked_add(listing)?
        .checked_add(u64::from(limits.printer.record).checked_mul(2)?)?
        .checked_add(queues)?
        .checked_add(u64::from(limits.mmdebstrap.path).checked_mul(8)?)
}
