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

#[test]
fn a_hunks_request_spawns_gits_diff_beneath_the_stores_root() {
    let mut layer = Layer::new();
    layer.hunks();
    let Some(Asked::Io(io::Request::Spawn { spawn, .. })) = layer.asked() else { panic!("diff spawn") };
    assert_eq!(spawn.program.as_ref(), b"/usr/bin/git");
    assert_eq!(spawn.root, skein_io::kernel::Fd::new(3));
    assert_eq!(spawn.dir.as_ref(), b"");
    assert_eq!(
        spawn.args.as_ref(),
        &[
            bytes::copy_of(b"--git-dir=/dev/null"),
            bytes::copy_of(b"diff"),
            bytes::copy_of(b"--no-index"),
            bytes::copy_of(b"--no-color"),
            bytes::copy_of(b"--no-ext-diff"),
            bytes::copy_of(b"--no-textconv"),
            bytes::copy_of(b"--diff-algorithm=myers"),
            bytes::copy_of(b"--unified=3"),
            bytes::copy_of(b"--"),
            bytes::copy_of(b"./cache/0101010101010101010101010101010101010101010101010101010101010101"),
            bytes::copy_of(b"./cache/0202020202020202020202020202020202020202020202020202020202020202"),
        ]
    );
}
const DIFF_FAILED_EXITS: &[Exit] = &[Exit::Code(0), Exit::Code(2), Exit::Code(128), Exit::Signal(9)];
#[test]
fn diff_exiting_0_or_past_1_fails_the_request_as_operational() {
    for exit in DIFF_FAILED_EXITS {
        let mut layer = Layer::new();
        layer.hunks();
        assert_eq!(
            layer.run(b"@@ -1 +1 @@\n-old\n+new\n", *exit),
            RootEvent::Render(Event::Failed { request: REQUEST, class: crate::render::vocabulary::Class::Operational }),
            "{exit:?}"
        );
    }
}
#[test]
fn diff_exiting_1_answers_its_decoded_hunks() {
    let mut layer = Layer::new();
    layer.hunks();
    assert_eq!(
        layer.run(b"@@ -1 +1 @@\n-old\n+new\n", Exit::Code(1)),
        RootEvent::Render(Event::Hunks {
            request: REQUEST,
            hunks: crate::render::diff::Hunks {
                lines: Box::new([
                    crate::render::diff::Line {
                        kind: crate::render::diff::LineKind::Hunk,
                        text: bytes::copy_of(b"@@ -1 +1 @@")
                    },
                    crate::render::diff::Line {
                        kind: crate::render::diff::LineKind::Removed,
                        text: bytes::copy_of(b"old")
                    },
                    crate::render::diff::Line {
                        kind: crate::render::diff::LineKind::Added,
                        text: bytes::copy_of(b"new")
                    },
                ]),
                left_out: 0,
                binary: false,
            }
        })
    );
}
#[test]
fn binary_diff_exiting_1_answers_no_hunk_line() {
    let mut layer = Layer::new();
    layer.hunks();
    assert_eq!(
        layer.run(b"Binary files a/a and b/b differ\n", Exit::Code(1)),
        RootEvent::Render(Event::Hunks {
            request: REQUEST,
            hunks: crate::render::diff::Hunks { lines: Box::new([]), left_out: 0, binary: true },
        })
    );
}

#[test]
fn hunks_wait_for_the_childs_exit_after_stdout_ends() {
    use skein_lib::stream::Held;
    let mut layer = Layer::new();
    layer.hunks();
    let Some(Asked::Io(io::Request::Spawn { owner, .. })) = layer.asked() else { panic!("diff spawn") };
    layer.spawn_pending(owner, false);
    let mut held = Held::new(bytes::copy_of(b"Binary files a/a and b/b differ\n"));
    for _ in 0_u32..64 {
        let Some(request) = layer.asked() else { break };
        layer.answer(request, &mut held, Exit::Code(1));
    }
    assert_eq!(layer.told(), None, "stdout's result waits for exit");
    layer.up(io::Event::Exited { owner, exit: Exit::Code(1) });
    assert_eq!(
        layer.run(b"", Exit::Code(1)),
        RootEvent::Render(Event::Hunks {
            request: REQUEST,
            hunks: crate::render::diff::Hunks { lines: Box::new([]), left_out: 0, binary: true }
        })
    );
}
