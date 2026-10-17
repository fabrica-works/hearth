use crate::{records::errors as records, render::errors as render};
use skein_lib::bytes::copy_of;
pub fn collision_back(value: records::Collision) -> render::Collision {
    match value {
        records::Collision::Key { key } => render::Collision::Key { key },
        records::Collision::Given { subject, name, first, second } => {
            render::Collision::Given { subject: subject_back(subject), name, first, second }
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
    }
}
#[must_use]
pub fn collision_faults() -> [render::Collision; 11] {
    [
        render::Collision::Key { key: copy_of(b"ops") },
        render::Collision::Key { key: copy_of(b"dev") },
        given(render::Subject::Tree),
        given(render::Subject::DataName),
        given(render::Subject::DataPath),
        given(render::Subject::Secret),
        given(render::Subject::Identity),
        given(render::Subject::Download),
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
