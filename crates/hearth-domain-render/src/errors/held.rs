//! The bytes an error owns, and the stable order of render.md, 7.1.
use super::{Cause, Collision, Composition, Decode, Error, Host, Kind, Order, Place, Repository, Resolution, Schema};
use alloc::boxed::Box;
use core::cmp::Ordering;

fn path(error: &Error) -> &[u8] {
    match &error.at {
        Some(place) => &place.path,
        None => b"",
    }
}
fn position(error: &Error) -> (u32, u32) {
    match &error.at {
        Some(place) => match place.position {
            Some(position) => (position.line, position.column),
            None => (0, 0),
        },
        None => (0, 0),
    }
}
pub(super) fn compare(first: &Error, second: &Error) -> Ordering {
    first
        .at
        .is_none()
        .cmp(&second.at.is_none())
        .then(path(first).cmp(path(second)))
        .then(position(first).cmp(&position(second)))
        .then(first.target.cmp(&second.target))
}
pub(super) fn past(error: &Error, cut: &Order) -> bool {
    !error
        .at
        .is_none()
        .cmp(&cut.unplaced)
        .then(path(error).cmp(&cut.path))
        .then(position(error).cmp(&cut.position))
        .then(error.target.cmp(&cut.target))
        .is_lt()
}
pub(super) fn order(error: &Error) -> Order {
    Order {
        unplaced: error.at.is_none(),
        path: skein_lib::bytes::copy_of(path(error)),
        position: position(error),
        target: error.target.clone(),
    }
}
pub(super) fn bytes(error: &Error) -> u64 {
    place(error.at.as_ref())
        .saturating_add(place(error.also.as_ref()))
        .saturating_add(optional(error.target.as_deref()))
        .saturating_add(optional(error.task.as_deref()))
        .saturating_add(kind(&error.kind))
}
fn size(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).expect("owned bytes fit u64")
}
fn place(place: Option<&Place>) -> u64 {
    match place {
        Some(place) => size(&place.path),
        None => 0,
    }
}
fn optional(bytes: Option<&[u8]>) -> u64 {
    match bytes {
        Some(bytes) => size(bytes),
        None => 0,
    }
}
fn strings(values: &[Box<[u8]>]) -> u64 {
    let mut bytes = u64::try_from(size_of_val(values)).expect("owned slice fits u64");
    for value in values {
        bytes = bytes.saturating_add(size(value));
    }
    bytes
}
fn kind(kind: &Kind) -> u64 {
    match kind {
        Kind::Reserved(value) => size(&value.what),
        Kind::Exported(value) => {
            size(&value.exporter).saturating_add(size(&value.task)).saturating_add(cause(&value.cause))
        }
        Kind::Ids(value) => size(&value.owner).saturating_add(size(&value.entry)),
        Kind::Resolution(value) => resolution(value),
        Kind::Guard(value) => size(&value.name).saturating_add(size(&value.message)),
        Kind::Collision(value) => collision(value),
        Kind::Store(value) => size(&value.store),
        Kind::Evaluation(_) => 0,
        Kind::Schema(value) => schema(value),
        Kind::Decode(value) => decode(value),
        Kind::Repository(value) => repository(value),
        Kind::Tool(value) => {
            let bytes = match &value.ended {
                super::Ended::Missing { oid } => size(&oid.0),
                super::Ended::Timeout
                | super::Ended::Start
                | super::Ended::Exit(_)
                | super::Ended::Signal(_)
                | super::Ended::Output
                | super::Ended::Stream => 0,
            };
            bytes.saturating_add(strings(&value.tail))
        }
        Kind::Composition(value) => composition(value),
    }
}
fn decode(value: &Decode) -> u64 {
    match value {
        Decode::Duplicate { key } => size(key),
        Decode::Settings { task } => size(task),
        Decode::Unknown { field } => size(field),
        Decode::Syntax(_)
        | Decode::Declaration(_)
        | Decode::ListNull
        | Decode::Unexpected
        | Decode::Trailing
        | Decode::TooDeep
        | Decode::StringTooLong
        | Decode::NumberTooLong
        | Decode::Number
        | Decode::Escape
        | Decode::Surrogate
        | Decode::Utf8
        | Decode::Control
        | Decode::Truncated
        | Decode::NotObject
        | Decode::NotInteger
        | Decode::Range
        | Decode::Nul
        | Decode::Keys
        | Decode::Note
        | Decode::Name
        | Decode::Placement
        | Decode::Members
        | Decode::Member
        | Decode::Wrong { .. }
        | Decode::Missing { .. }
        | Decode::Misplaced { .. }
        | Decode::TooLarge { .. } => 0,
    }
}
fn repository(value: &Repository) -> u64 {
    match value {
        Repository::Environment { env } => size(env),
        Repository::NoTree
        | Repository::NoTemplate
        | Repository::NoScript
        | Repository::Unused
        | Repository::LinkTarget
        | Repository::Stray
        | Repository::Name
        | Repository::Declaration
        | Repository::Static
        | Repository::Destination
        | Repository::Submodule
        | Repository::Link => 0,
    }
}
fn composition(value: &Composition) -> u64 {
    match value {
        Composition::On { on, host } => {
            let bytes = match host {
                Host::Elsewhere { environment } => size(environment),
                Host::Unplaced | Host::Unknown | Host::Container => 0,
            };
            size(on).saturating_add(bytes)
        }
        Composition::Duplicate
        | Composition::Long
        | Composition::Placement
        | Composition::Members
        | Composition::Unplaced
        | Composition::Null
        | Composition::Task => 0,
    }
}
fn schema(value: &Schema) -> u64 {
    match value {
        Schema::Source { .. }
        | Schema::Export { .. }
        | Schema::Dynamic { .. }
        | Schema::Wrong { .. }
        | Schema::Below { .. }
        | Schema::Above { .. }
        | Schema::Length { .. }
        | Schema::Null
        | Schema::Malformed { .. }
        | Schema::Main
        | Schema::Security => 0,
        Schema::Unlisted { reader, task, setting, .. } => {
            size(reader).saturating_add(size(task)).saturating_add(optional(setting.as_deref()))
        }
        Schema::Task { task } | Schema::Absent { task } => size(task),
        Schema::Unread { task, setting, declared } => {
            size(task).saturating_add(size(setting)).saturating_add(strings(declared))
        }
        Schema::Enum { value, listed } => size(value).saturating_add(strings(listed)),
        Schema::Beneath { allowed } => strings(allowed),
        Schema::UndeclaredField { field, declared } => size(field).saturating_add(strings(declared)),
        Schema::MissingField { field } => size(field),
        Schema::Undeclared { setting, declared } => size(setting).saturating_add(strings(declared)),
        Schema::Missing { setting } => size(setting),
        Schema::Target { name, environment } => size(name).saturating_add(size(environment)),
        Schema::File { path } | Schema::Large { path, .. } => size(path),
        Schema::Component { package, component } => size(package).saturating_add(size(component)),
    }
}
fn cause(value: &Cause) -> u64 {
    match value {
        Cause::Decode(value) => decode(value),
        Cause::Repository(value) => repository(value),
        Cause::Composition(value) => composition(value),
        Cause::Schema(value) => schema(value),
        Cause::Evaluation(_) => 0,
    }
}
fn collision(value: &Collision) -> u64 {
    match value {
        Collision::Key { key } => size(key),
        Collision::Given { name, first, second, .. } | Collision::Differs { name, first, second, .. } => {
            size(name).saturating_add(size(first)).saturating_add(size(second))
        }
        Collision::Shared { first, second, .. } | Collision::Overlap { first, second, .. } => {
            size(first).saturating_add(size(second))
        }
        Collision::Hidden { path, data, first, second } => {
            size(path).saturating_add(size(data)).saturating_add(size(first)).saturating_add(size(second))
        }
    }
}
fn resolution(value: &Resolution) -> u64 {
    match value {
        Resolution::TooLarge { .. } | Resolution::TooMany { .. } => 0,
        Resolution::Unknown { package } | Resolution::Missing { package } => size(package),
        Resolution::Unsatisfiable { package } => optional(package.as_deref()),
        Resolution::Absent { uri, suite } => size(uri).saturating_add(size(suite)),
        Resolution::Unmatched { name, version, architecture }
        | Resolution::Disagree { name, version, architecture } => {
            size(name).saturating_add(size(version)).saturating_add(size(architecture))
        }
    }
}
