use super::{File, Laid, Rendered, Repository};
use hearth::Form;
use hearth_lib::encode::{Digest, Item};
use hearth_protocol_tree::resolution::Package;
const VIM_KEY: Digest = Digest([
    0xfe, 0x02, 0x39, 0xc3, 0x52, 0x51, 0x49, 0xdb, 0xf1, 0x9e, 0x27, 0x76, 0x80, 0x30, 0xd8, 0xe6, 0x1b, 0x45, 0xe7,
    0x7a, 0x04, 0x57, 0x8c, 0xe1, 0x49, 0xe2, 0xea, 0x86, 0xcc, 0x75, 0x72, 0x42,
]);
const BEFORE_CLOSURE: Digest = Digest([
    0xdc, 0xc9, 0xcb, 0x83, 0x25, 0xeb, 0xe2, 0x7a, 0x4e, 0x87, 0xcd, 0xf6, 0xe8, 0x49, 0x1b, 0x04, 0x42, 0x50, 0x15,
    0xb9, 0xc6, 0x64, 0x19, 0x34, 0xc4, 0x3f, 0xfa, 0xb0, 0x85, 0xd9, 0x7b, 0xad,
]);
const AFTER_CLOSURE: Digest = Digest([
    0x84, 0x59, 0x33, 0x5e, 0xbe, 0xb4, 0x41, 0xd4, 0x15, 0xb6, 0xe4, 0x5b, 0xc8, 0x4a, 0x42, 0x3c, 0x4e, 0x18, 0x7c,
    0x2a, 0xfa, 0xff, 0x82, 0x8f, 0x51, 0x74, 0x72, 0x89, 0x76, 0x49, 0x25, 0xcb,
]);

fn package(name: &[u8], version: &[u8]) -> Package {
    Package {
        name: name.into(),
        architecture: b"amd64".as_slice().into(),
        version: version.into(),
        source: 0,
        filename: b"pool/a.deb".as_slice().into(),
        size: 1,
        sha256: Digest([1; 32]),
    }
}
#[must_use]
pub fn recorded_resolution_diff(label: &str, form: Form) -> Rendered {
    let repository = Repository::make(label, super::TEST_REPOSITORY);
    let from = repository.head();
    let to = repository.next(
        &[
            File { path: "tasks/extra/task.json", body: br#"{"packages":{"vim":{}}}"#, mode: 0o644 },
            File { path: "hosts/test/test-min.json", body: br#"{"extra":{}}"#, mode: 0o644 },
        ],
        &[],
    );
    let mut before = super::test_min_recorded();
    before.closure = Box::new([package(b"apt", b"2"), package(b"nano", b"8")]);
    before.digest = BEFORE_CLOSURE;
    let mut after = before.clone();
    after.key = VIM_KEY;
    after.inputs.request = Box::new([Item::List(1), Item::Bytes(b"vim".as_slice().into())]);
    after.closure = Box::new([package(b"apt", b"3"), package(b"vim", b"9")]);
    after.digest = AFTER_CLOSURE;
    super::lay(&repository.scratch().path().join("store"), &[Laid::Resolution(before), Laid::Resolution(after)]);
    super::diff(&repository, &super::Asked { form, targets: &[b"test-min"], ..super::Asked::new(&from, &to) })
}
