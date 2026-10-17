use crate::revision::{Class, class};
const PATHS: &[(&[u8], Class<'_>)] = &[
    (b"fleet.json", Class::Fleet),
    (b"envs/prod.json", Class::Environment { env: b"prod" }),
    (b"hosts/x1/x1.json", Class::Host { base: b"x1", name: b"x1" }),
    (b"hosts/x1/x1-prod.json", Class::Host { base: b"x1", name: b"x1-prod" }),
    (b"hosts/x1/deep/a.json", Class::Other),
    (b"envs/a/b.json", Class::Other),
    (b"envs/.json", Class::Other),
    (b"hosts//x1.json", Class::Other),
    (b"tasks/motd/task.json", Class::Declaration { task: b"motd" }),
    (b"tasks/motd/tree/etc/motd", Class::Tree { task: b"motd", path: b"etc/motd" }),
    (b"README.md", Class::Other),
];
#[test]
fn paths_are_classed_as_format_md_2_1_lays_them_out() {
    for &(path, expected) in PATHS {
        assert_eq!(class(path), expected, "path {path:?}");
    }
}

#[test]
fn the_first_read_asks_every_listed_task_json_once() {
    let call = super::fixtures::declaration_call();
    assert_eq!(
        call.pending.as_ref(),
        &[
            crate::vocabulary::Blob {
                oid: crate::revision::Oid(skein_lib::bytes::copy_of(b"core")),
                size: 2,
                kind: crate::vocabulary::As::Declaration
            },
            crate::vocabulary::Blob {
                oid: crate::revision::Oid(skein_lib::bytes::copy_of(b"motd")),
                size: 2,
                kind: crate::vocabulary::As::Declaration
            },
        ]
    );
}
#[test]
fn the_second_read_never_asks_a_task_json_again() {
    let mut call = super::fixtures::declaration_call();
    for name in [b"core".as_slice(), b"motd"] {
        call.decoded(&crate::revision::Oid(skein_lib::bytes::copy_of(name)), crate::vocabulary::Decoded::Declaration);
    }
    let mut out = skein_lib::Queue::with_capacity(1);
    assert!(!call.files(crate::limits::SHIPPED, &mut out));
    assert_eq!(out.pop(), None);
}
