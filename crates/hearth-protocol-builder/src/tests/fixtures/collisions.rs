use crate::{records::errors as records, render::errors as render};
use skein_lib::bytes::copy_of;
pub fn collision_back(value: records::Collision) -> render::Collision {
    match value {
        records::Collision::Key { key } => render::Collision::Key { key },
        records::Collision::Given { subject, name, first, second } => {
            render::Collision::Given { subject: subject_back(subject), name, first, second }
        }
        records::Collision::Differs { subject, name, first, second } => {
            render::Collision::Differs { subject: subject_back(subject), name, first, second }
        }
        records::Collision::Shared { id, value, first, second } => {
            render::Collision::Shared { id: id_back(id), value, first, second }
        }
        records::Collision::Hidden { path, data, first, second } => {
            render::Collision::Hidden { path, data, first, second }
        }
        records::Collision::Overlap { subject, first, second } => {
            render::Collision::Overlap { subject: subject_back(subject), first, second }
        }
    }
}
fn subject_back(value: records::Subject) -> render::Subject {
    match value {
        records::Subject::Tree => render::Subject::Tree,
        records::Subject::DataName => render::Subject::DataName,
        records::Subject::DataPath => render::Subject::DataPath,
        records::Subject::Secret => render::Subject::Secret,
        records::Subject::Identity => render::Subject::Identity,
        records::Subject::Download => render::Subject::Download,
        records::Subject::User => render::Subject::User,
        records::Subject::Group => render::Subject::Group,
        records::Subject::Preset => render::Subject::Preset,
        records::Subject::Tmpfiles => render::Subject::Tmpfiles,
        records::Subject::Preseed => render::Subject::Preseed,
        records::Subject::Parameter => render::Subject::Parameter,
        records::Subject::Range => render::Subject::Range,
    }
}
#[must_use]
pub fn collision_faults() -> [render::Collision; 22] {
    [
        render::Collision::Key { key: copy_of(b"ops") },
        render::Collision::Key { key: copy_of(b"dev") },
        given(render::Subject::Tree),
        given(render::Subject::DataName),
        given(render::Subject::DataPath),
        given(render::Subject::Secret),
        given(render::Subject::Identity),
        given(render::Subject::Download),
        differs(render::Subject::User),
        differs(render::Subject::Group),
        differs(render::Subject::Preset),
        differs(render::Subject::Tmpfiles),
        differs(render::Subject::Preseed),
        differs(render::Subject::Parameter),
        differs(render::Subject::Range),
        shared(render::Id::Uid),
        shared(render::Id::Gid),
        render::Collision::Overlap {
            subject: render::Subject::Range,
            first: copy_of(b"alice"),
            second: copy_of(b"bob"),
        },
        render::Collision::Differs {
            subject: render::Subject::User,
            name: copy_of(b"free"),
            first: copy_of(b"first"),
            second: copy_of(b"second"),
        },
        render::Collision::Hidden {
            path: copy_of(b"/srv/app/seed"),
            data: copy_of(b"/srv/app"),
            first: copy_of(b"data"),
            second: copy_of(b"tree"),
        },
        render::Collision::Overlap {
            subject: render::Subject::DataPath,
            first: copy_of(b"/srv"),
            second: copy_of(b"/srv/app"),
        },
        render::Collision::Overlap {
            subject: render::Subject::Download,
            first: copy_of(b"/opt"),
            second: copy_of(b"/opt/app"),
        },
    ]
}
fn given(subject: render::Subject) -> render::Collision {
    render::Collision::Given { subject, name: copy_of(b"name"), first: copy_of(b"a"), second: copy_of(b"b") }
}
fn differs(subject: render::Subject) -> render::Collision {
    render::Collision::Differs { subject, name: copy_of(b"name"), first: copy_of(b"a"), second: copy_of(b"b") }
}
fn id_back(id: records::Id) -> render::Id {
    match id {
        records::Id::Uid => render::Id::Uid,
        records::Id::Gid => render::Id::Gid,
    }
}
// A valid account id inside format.md 5.6's bounds.
pub const COLLISION_UID: u32 = 1_000;
fn shared(id: render::Id) -> render::Collision {
    render::Collision::Shared { id, value: COLLISION_UID, first: copy_of(b"alice"), second: copy_of(b"bob") }
}
