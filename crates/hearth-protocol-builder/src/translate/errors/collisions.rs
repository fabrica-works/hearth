//! Collision vocabulary has the same typed fields at both boundaries.
use super::{records, target};
pub(super) fn collision(value: target::Collision) -> records::errors::Collision {
    use records::errors::Collision;
    match value {
        target::Collision::Key { key } => Collision::Key { key },
        target::Collision::Given { subject: value, name, first, second } => {
            Collision::Given { subject: subject(value), name, first, second }
        }
        target::Collision::Differs { subject: value, name, first, second } => {
            Collision::Differs { subject: subject(value), name, first, second }
        }
        target::Collision::Shared { id, value, first, second } => {
            Collision::Shared { id: id_to_records(id), value, first, second }
        }
        target::Collision::Hidden { path, data, first, second } => Collision::Hidden { path, data, first, second },
        target::Collision::Overlap { subject: value, first, second } => {
            Collision::Overlap { subject: subject(value), first, second }
        }
    }
}
fn subject(value: target::Subject) -> records::errors::Subject {
    use records::errors::Subject;
    match value {
        target::Subject::Tree => Subject::Tree,
        target::Subject::DataName => Subject::DataName,
        target::Subject::DataPath => Subject::DataPath,
        target::Subject::Secret => Subject::Secret,
        target::Subject::Identity => Subject::Identity,
        target::Subject::Download => Subject::Download,
        target::Subject::User => Subject::User,
        target::Subject::Group => Subject::Group,
        target::Subject::Preset => Subject::Preset,
        target::Subject::Tmpfiles => Subject::Tmpfiles,
        target::Subject::Preseed => Subject::Preseed,
        target::Subject::Parameter => Subject::Parameter,
        target::Subject::Range => Subject::Range,
    }
}
fn id_to_records(id: target::Id) -> records::errors::Id {
    match id {
        target::Id::Uid => records::errors::Id::Uid,
        target::Id::Gid => records::errors::Id::Gid,
    }
}
