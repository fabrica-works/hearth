use super::{bytes, composition, limits};
pub const TEMPLATE_ENTRIES: u32 = 5; // Four removed_task entries and one template.
pub const SHARED_TEMPLATE_ENTRIES: u32 = TEMPLATE_ENTRIES + 1;

pub fn decode_error(
    path: &[u8],
    position: Option<crate::revision::Position>,
    decode: crate::errors::Decode,
    task: Option<Box<[u8]>>,
) -> crate::errors::Error {
    crate::errors::Error {
        side: None,
        kind: crate::errors::Kind::Decode(decode),
        class: crate::errors::Class::Configuration,
        at: Some(crate::errors::Place { path: bytes::copy_of(path), position }),
        also: None,
        target: None,
        task,
    }
}

pub const SCHEMA_FAULTS: u32 = 16; // The schema validator's bounded diagnostics per task.
pub const DOCUMENT_BYTES: u32 = 16; // Below error_call's 24-byte documents.
pub const REVISION_BYTES: u64 = 132; // Two decoded 66-byte maps fit; the third crosses.
pub const STATIC_BYTES: u32 = 8; // The oversized static fixture is one byte larger.
pub const SINGLE_REQUEST: u32 = 1; // The call asks one read at a time.
pub const TWO_ERRORS: u32 = 2;
pub const THREE_ERRORS: u32 = TWO_ERRORS + 1;
pub const EIGHT_ERRORS: u32 = 8; // Holds every row of the four-row sorting fixture.

/// A place and target, using zero coordinates for a whole file or no place.
pub type Spot<'a> = (&'a [u8], u32, u32, &'a [u8]);
pub fn limits_with_errors(errors: u32) -> limits::Limits {
    limits::Limits { errors, ..limits::SHIPPED }
}
pub fn error_at(path: &[u8], line: u32, column: u32, target: &[u8]) -> crate::errors::Error {
    let mut error =
        crate::errors::refused(path, Some(composition::at(line, column)), crate::errors::Decode::Unexpected, None);
    error.target = Some(bytes::copy_of(target));
    error
}
pub fn unplaced(target: &[u8]) -> crate::errors::Error {
    let mut error = error_at(b"", 0, 0, target);
    error.at = None;
    error
}
pub fn collector(bound: u32) -> crate::errors::Collector {
    let mut collector = crate::errors::Collector::new(&limits_with_errors(bound));
    for path in [b"d", b"b", b"a", b"c"] {
        collector.push(error_at(path, 1, 1, b"t"));
    }
    collector
}
pub fn kept(call: &crate::call::Side) -> Box<[crate::errors::Error]> {
    call.errors.errors()
}
pub fn places(errors: &[crate::errors::Error]) -> Box<[Spot<'_>]> {
    let mut places = skein_lib::List::with_capacity(u32::try_from(errors.len()).expect("test errors"));
    for error in errors {
        let (path, line, column) = match &error.at {
            Some(place) => match place.position {
                Some(position) => (place.path.as_ref(), position.line, position.column),
                None => (place.path.as_ref(), 0, 0),
            },
            None => (b"".as_slice(), 0, 0),
        };
        places.push((path, line, column, error.target.as_deref().unwrap_or(b""))).expect("error count");
    }
    places.into_boxed()
}

// render.md, 7.1: 16 MiB across the strings of the retained errors.
pub const ERROR_BYTES: usize = 16 * 1024 * 1024;
pub const HALF_ERROR_BYTES: usize = ERROR_BYTES / 2;
pub const NO_ERRORS: u32 = 0;

pub fn large_error(path: &[u8], bytes: usize) -> crate::errors::Error {
    let mut error = error_at(path, 1, 1, b"");
    error.kind = crate::errors::Kind::Guard(crate::errors::Guard {
        name: Box::new([]),
        message: bytes::zeroed(bytes.saturating_sub(path.len())),
    });
    error
}
pub fn byte_limited_errors() -> crate::errors::Collector {
    let mut errors = super::composition_collector();
    errors.push(large_error(b"d", 1));
    errors.push(large_error(b"c", HALF_ERROR_BYTES + 1));
    errors.push(large_error(b"b", HALF_ERROR_BYTES));
    errors.push(large_error(b"a", HALF_ERROR_BYTES));
    errors
}
