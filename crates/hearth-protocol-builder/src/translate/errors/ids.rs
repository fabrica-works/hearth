//! Fixed-owner errors cross the client boundary without losing the entry.
use crate::{records::errors as target, render::errors as source};
pub(super) fn ids(value: source::Ids) -> target::Ids {
    let source::Ids { owner, of, entry, why } = value;
    target::Ids { owner, of: owns(of), entry, why: reason(why) }
}
fn owns(value: source::Owns) -> target::Owns {
    match value {
        source::Owns::Data => target::Owns::Data,
        source::Owns::Secret => target::Owns::Secret,
        source::Owns::Identity => target::Owns::Identity,
        source::Owns::Download => target::Owns::Download,
        source::Owns::User => target::Owns::User,
    }
}
fn reason(value: source::Why) -> target::Why {
    match value {
        source::Why::NoUser => target::Why::NoUser,
        source::Why::NoGroup => target::Why::NoGroup,
        source::Why::NoUid => target::Why::NoUid,
        source::Why::NoGid => target::Why::NoGid,
        source::Why::OneId => target::Why::OneId,
        source::Why::NotPackageUser => target::Why::NotPackageUser,
        source::Why::NotPackageGroup => target::Why::NotPackageGroup,
    }
}
