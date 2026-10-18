use crate::errors::{Class, Collision, Error, Kind, Place, Position, Subject};
use skein_lib::bytes::copy_of;
pub fn path_collision() -> Error {
    Error {
        side: None,
        kind: Kind::Collision(Collision::Given {
            subject: Subject::Tree,
            name: copy_of(b"/etc/motd"),
            first: copy_of(b"banner"),
            second: copy_of(b"motd"),
        }),
        class: Class::Configuration,
        at: Some(Place { path: copy_of(b"tasks/banner/task.json"), position: Some(Position { line: 1, column: 12 }) }),
        also: Some(Place { path: copy_of(b"tasks/motd/tree/etc/motd"), position: None }),
        target: Some(copy_of(b"x1-prod")),
        task: None,
    }
}
// A declared account id inside format.md 5.6's bounds.
pub const COLLISION_UID: u32 = 1_000;
pub fn shared_uid_collision() -> Error {
    let mut error = path_collision();
    error.kind = Kind::Collision(Collision::Shared {
        id: crate::errors::Id::Uid,
        value: COLLISION_UID,
        first: copy_of(b"free"),
        second: copy_of(b"ci"),
    });
    error
}
pub fn range_collision() -> Error {
    let mut error = path_collision();
    error.kind = Kind::Collision(Collision::Overlap {
        subject: Subject::Range,
        first: copy_of(b"free"),
        second: copy_of(b"ci"),
    });
    error
}
