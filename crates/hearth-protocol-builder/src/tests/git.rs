use super::fixtures::{Layer, REQUEST};
use crate::{
    Asked,
    builder::{
        render::{
            revision::{Entry, Mode},
            vocabulary::Event,
        },
        vocabulary::Event as RootEvent,
    },
};
use skein_io::{self as io, kernel::Exit};
use skein_lib::bytes;

#[test]
fn a_list_spawns_rev_parse_with_render_md_2_2s_vector_and_environment() {
    let mut layer = Layer::new();
    layer.list();
    let Some(Asked::Io(io::Request::Spawn { spawn, .. })) = layer.asked() else {
        panic!("rev-parse spawn");
    };
    assert_eq!(&*spawn.program, b"/usr/bin/git", "render.md 2.2 executable");
    assert_eq!(
        spawn.args.as_ref(),
        &[
            bytes::copy_of(b"--git-dir=/repo"),
            bytes::copy_of(b"rev-parse"),
            bytes::copy_of(b"--verify"),
            bytes::copy_of(b"--quiet"),
            bytes::copy_of(b"HEAD^{commit}")
        ],
        "rev-parse vector"
    );
    assert_eq!(
        spawn.env.as_ref(),
        &[
            bytes::copy_of(b"LC_ALL=C"),
            bytes::copy_of(b"GIT_CONFIG_NOSYSTEM=1"),
            bytes::copy_of(b"GIT_CONFIG_GLOBAL=/dev/null"),
            bytes::copy_of(b"GIT_NO_REPLACE_OBJECTS=1"),
            bytes::copy_of(b"GIT_TERMINAL_PROMPT=0"),
            bytes::copy_of(b"HOME=/dev/null")
        ],
        "complete environment"
    );
}
#[test]
fn rev_parse_exiting_1_answers_unresolved() {
    let mut layer = Layer::new();
    layer.list();
    assert_eq!(
        layer.run(b"", Exit::Code(1)),
        RootEvent::Render(Event::Unresolved { request: REQUEST }),
        "unresolved revision"
    );
}
#[test]
fn ls_tree_uses_the_commit_rev_parse_named() {
    let mut layer = Layer::new();
    let Asked::Io(io::Request::Spawn { spawn, .. }) = layer.resolve() else {
        panic!("listing spawn");
    };
    assert_eq!(
        spawn.args.as_ref(),
        &[
            bytes::copy_of(b"--git-dir=/repo"),
            bytes::copy_of(b"ls-tree"),
            bytes::copy_of(b"-r"),
            bytes::copy_of(b"-l"),
            bytes::copy_of(b"-z"),
            bytes::copy_of(b"--full-tree"),
            bytes::copy_of(b"0123456789abcdef0123456789abcdef01234567")
        ],
        "resolved commit vector"
    );
}
#[test]
fn the_commit_rev_parse_names_is_listed_with_git_s_entries() {
    let mut layer = Layer::new();
    let Asked::Io(io::Request::Spawn { owner, .. }) = layer.resolve() else {
        panic!("listing spawn");
    };
    layer.spawned(owner, false, Exit::Code(0));
    assert_eq!(
        layer.run(b"100755 blob 0123456789abcdef0123456789abcdef01234567       2\tenvs/prod.json\0", Exit::Code(0)),
        RootEvent::Render(Event::Listed {
            request: REQUEST,
            commit: crate::render::revision::Oid(bytes::copy_of(b"0123456789abcdef0123456789abcdef01234567")),
            entries: Box::new([Entry {
                path: bytes::copy_of(b"envs/prod.json"),
                mode: Mode::Executable,
                oid: crate::render::revision::Oid(bytes::copy_of(b"0123456789abcdef0123456789abcdef01234567")),
                size: 2
            }])
        }),
        "git entries reach render"
    );
}
const FAILED_EXITS: &[Exit] = &[Exit::Code(2), Exit::Code(128), Exit::Signal(9)];
#[test]
fn git_exiting_past_its_success_fails_the_request_as_operational() {
    for exit in FAILED_EXITS {
        let mut layer = Layer::new();
        layer.list();
        assert_eq!(
            layer.run(b"", *exit),
            RootEvent::Render(Event::Failed { request: REQUEST, class: crate::render::vocabulary::Class::Operational }),
            "exit {exit:?}"
        );
    }
}
#[test]
fn a_failed_spawn_waits_for_closed_before_answering_operational() {
    let mut layer = Layer::new();
    layer.list();
    let Some(Asked::Io(io::Request::Spawn { owner, .. })) = layer.asked() else {
        panic!("spawn");
    };
    layer.up(io::Event::Failed { owner, error: io::Error::Other });
    assert_eq!(layer.asked(), None, "Failed still owes Closed");
    assert_eq!(layer.told(), None, "no answer before Closed");
    layer.up(io::Event::Closed { owner });
    assert_eq!(layer.asked(), None, "cleanup emits no further request");
    assert_eq!(
        layer.told(),
        Some(RootEvent::Render(Event::Failed {
            request: REQUEST,
            class: crate::render::vocabulary::Class::Operational
        })),
        "spawn failed"
    );
}
