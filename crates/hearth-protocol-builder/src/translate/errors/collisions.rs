//! Collision vocabulary has the same typed fields at both boundaries.
use super::{records, target};
pub(super) fn collision(value: target::Collision) -> records::errors::Collision {
    use records::errors::Collision;
    match value {
        target::Collision::Key { key } => Collision::Key { key },
        target::Collision::Given { subject: value, name, first, second } => {
            Collision::Given { subject: subject(value), name, first, second }
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
    }
}
