use crate::render;
use alloc::boxed::Box;
use skein_lib::bytes;
pub const MIN_UID: i64 = 1000;
pub const MAX_UID: i64 = 59999;
#[must_use]
pub fn schema_faults() -> [render::errors::Schema; 22] {
    use render::errors::Schema;
    [
        Schema::Undeclared { setting: bytes::copy_of(b"greting"), declared: Box::new([bytes::copy_of(b"greeting")]) },
        Schema::Missing { setting: bytes::copy_of(b"greeting") },
        Schema::Absent { task: bytes::copy_of(b"wifi") },
        Schema::Wrong { expected: render::errors::TypeName::Int },
        Schema::Enum {
            value: bytes::copy_of(b"x"),
            listed: Box::new([bytes::copy_of(b"main"), bytes::copy_of(b"contrib")]),
        },
        Schema::Below { min: -MIN_UID },
        Schema::Above { max: MAX_UID },
        Schema::Length { max_length: 8 },
        Schema::Null,
        Schema::Malformed { format: render::errors::Format::Path },
        Schema::Target { name: bytes::copy_of(b"x9-prod"), environment: bytes::copy_of(b"prod") },
        Schema::File { path: bytes::copy_of(b"lab-ca/lab.crt") },
        Schema::Large { path: bytes::copy_of(b"lab-ca/big.crt"), max: 67_108_864 },
        Schema::Beneath { allowed: Box::new([bytes::copy_of(b"/etc"), bytes::copy_of(b"/opt")]) },
        Schema::Main,
        Schema::Security,
        Schema::Unlisted {
            reader: bytes::copy_of(b"vpn"),
            root: render::errors::Root::Tasks,
            task: bytes::copy_of(b"core"),
            setting: Some(bytes::copy_of(b"domain")),
        },
        Schema::Dynamic { root: render::errors::Root::Exports },
        Schema::UndeclaredField {
            field: bytes::copy_of(b"shel"),
            declared: Box::new([bytes::copy_of(b"shell"), bytes::copy_of(b"uid")]),
        },
        Schema::MissingField { field: bytes::copy_of(b"uid") },
        Schema::Task { task: bytes::copy_of(b"mail") },
        Schema::Unread {
            task: bytes::copy_of(b"core"),
            setting: bytes::copy_of(b"domian"),
            declared: Box::new([bytes::copy_of(b"domain")]),
        },
    ]
}
