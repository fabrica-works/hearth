use crate::revision::{Class, class};
const PATHS: &[(&[u8], Class<'_>)] = &[
    (b"fleet.json", Class::Fleet),
    (b"envs/prod.json", Class::Environment { env: b"prod" }),
    (b"hosts/x1/x1.json", Class::Host { base: b"x1", name: b"x1" }),
    (b"hosts/x1/x1-prod.json", Class::Host { base: b"x1", name: b"x1-prod" }),
    (b"hosts/x1/deep/a.json", Class::Other),
    (b"envs/a/b.json", Class::Refused { task: None, why: Repository::Name }),
    (b"envs/.json", Class::Refused { task: None, why: Repository::Name }),
    (b"hosts//x1.json", Class::Refused { task: None, why: Repository::Name }),
    (b"tasks/motd/task.json", Class::Declaration { task: b"motd" }),
    (b"tasks/motd/tree/etc/motd", Class::Tree { task: b"motd", path: b"etc/motd" }),
    (b"README.md", Class::Other),
];
#[test]
fn paths_are_classed_as_format_md_2_1_lays_them_out() {
    for (path, expected) in PATHS {
        assert_eq!(class(path), *expected, "path {path:?}");
    }
}

#[test]
fn the_first_read_asks_every_listed_task_json_once() {
    let call = fixtures::declaration_call();
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
    let mut call = fixtures::declaration_call();
    for name in [b"core".as_slice(), b"motd"] {
        call.decoded(
            &crate::revision::Oid(skein_lib::bytes::copy_of(name)),
            crate::vocabulary::Decoded::Declaration(fixtures::declaration(&[], &[])),
        );
    }
    let mut out = skein_lib::Queue::with_capacity(1);
    assert!(!call.files(limits::SHIPPED, &mut out));
    assert_eq!(out.pop(), None);
}

use super::fixtures;
use crate::{errors::Repository, limits, revision::Mode};
const REFUSED: &[(&[u8], Mode, Repository)] = &[
    (b"envs/lab/x.json", Mode::File, Repository::Name),
    (b"envs/Lab.json", Mode::File, Repository::Name),
    (b"envs/lab-.json", Mode::File, Repository::Name),
    (b"hosts/x1/notes.json", Mode::File, Repository::Stray),
    (b"hosts/X1/X1.json", Mode::File, Repository::Name),
    (b"tasks/ssh/tre/x", Mode::File, Repository::Stray),
    (b"tasks/ssh/templates/a.conf", Mode::File, Repository::Stray),
    (b"tasks/Ssh/task.json", Mode::File, Repository::Name),
    (b"fleet.json", Mode::Link, Repository::Link),
    (b"tasks/ssh/tree/x", Mode::Submodule, Repository::Submodule),
];
#[test]
fn paths_format_md_2_1_does_not_allow_are_refused() {
    for (path, mode, why) in REFUSED {
        let (_, errors) = fixtures::listing(&[(b"tasks/ssh/task.json", Mode::File), (path, *mode)], &limits::SHIPPED);
        assert_eq!(fixtures::only(errors.as_ref()).kind, crate::errors::Kind::Repository(why.clone()), "{path:?}");
    }
}
#[test]
fn a_base_env_without_its_environment_is_a_repository_error() {
    let (_, errors) = fixtures::listing(
        &[(b"hosts/x1/x1.json", Mode::File), (b"hosts/x1/x1-lab.json", Mode::File)],
        &limits::SHIPPED,
    );
    assert_eq!(
        fixtures::only(errors.as_ref()).kind,
        crate::errors::Kind::Repository(Repository::Environment { env: skein_lib::bytes::copy_of(b"lab") })
    );
}
#[test]
fn a_task_directory_without_task_json_is_refused() {
    let (_, errors) =
        fixtures::listing(&[(b"tasks/ssh/tree/x", Mode::File), (b"README.md", Mode::File)], &limits::SHIPPED);
    assert_eq!(fixtures::only(errors.as_ref()).kind, crate::errors::Kind::Repository(Repository::Declaration));
}
const DESTINATIONS: &[(&[u8], &[u8])] = &[
    (b"tasks/ssh/tree/etc/foo", b"tasks/ssh/tree/etc/foo.j2"),
    (b"tasks/ssh/customize.sh", b"tasks/ssh/customize.sh.j2"),
];
#[test]
fn two_sources_of_one_destination_err_at_the_second() {
    for &(first, second) in DESTINATIONS {
        let (_, errors) = fixtures::listing(
            &[(b"tasks/ssh/task.json", Mode::File), (second, Mode::File), (first, Mode::File)],
            &limits::SHIPPED,
        );
        let error = fixtures::only(errors.as_ref());
        assert_eq!(error.kind, crate::errors::Kind::Repository(Repository::Destination));
        assert_eq!(error.at.as_ref().expect("path").path.as_ref(), second);
        assert_eq!(error.also.as_ref().expect("first source").path.as_ref(), first);
    }
}
#[test]
fn the_task_past_the_limit_errs_and_fails_the_call() {
    let (valid, errors) = fixtures::listing(
        &[(b"tasks/a/task.json", Mode::File), (b"tasks/b/task.json", Mode::File), (b"tasks/c/task.json", Mode::File)],
        &limits::Limits { tasks: 2, ..limits::SHIPPED },
    );
    assert!(!valid);
    assert_eq!(
        fixtures::only(errors.as_ref()).kind,
        crate::errors::Kind::Decode(crate::errors::Decode::TooLarge { limit: crate::errors::Limit::Tasks, max: 2 })
    );
}
