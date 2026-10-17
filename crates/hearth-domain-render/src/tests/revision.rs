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
    (b"tasks/motd/task.json", Class::Other),
    (b"README.md", Class::Other),
];
#[test]
fn paths_are_classed_as_format_md_2_1_lays_them_out() {
    for &(path, expected) in PATHS {
        assert_eq!(class(path), expected, "path {path:?}");
    }
}
