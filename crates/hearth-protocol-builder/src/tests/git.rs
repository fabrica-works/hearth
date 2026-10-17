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
const FAILED_EXITS: &[(Exit, crate::render::errors::Ended)] = &[
    (Exit::Code(2), crate::render::errors::Ended::Exit(2)),
    (Exit::Code(128), crate::render::errors::Ended::Exit(128)),
    (Exit::Signal(9), crate::render::errors::Ended::Signal(9)),
];
#[test]
fn git_exiting_past_its_success_fails_the_request_as_operational() {
    for (exit, ended) in FAILED_EXITS {
        let mut layer = Layer::new();
        layer.list();
        assert_eq!(
            layer.run(b"", *exit),
            RootEvent::Render(Event::Failed {
                request: REQUEST,
                class: crate::render::vocabulary::Class::Operational,
                at: None,
                failure: crate::render::vocabulary::Failure::Tool(crate::render::errors::Tool {
                    command: crate::render::errors::Command::RevParse,
                    ended: ended.clone(),
                    tail: Box::new([])
                })
            }),
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
            class: crate::render::vocabulary::Class::Operational,
            at: None,
            failure: crate::render::vocabulary::Failure::Tool(crate::render::errors::Tool {
                command: crate::render::errors::Command::RevParse,
                ended: crate::render::errors::Ended::Start,
                tail: Box::new([])
            })
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
const DIFF_FAILED_EXITS: &[(Exit, crate::render::errors::Ended)] = &[
    (Exit::Code(0), crate::render::errors::Ended::Exit(0)),
    (Exit::Code(2), crate::render::errors::Ended::Exit(2)),
    (Exit::Code(128), crate::render::errors::Ended::Exit(128)),
    (Exit::Signal(9), crate::render::errors::Ended::Signal(9)),
];
#[test]
fn diff_exiting_0_or_past_1_fails_the_request_as_operational() {
    for (exit, ended) in DIFF_FAILED_EXITS {
        let mut layer = Layer::new();
        layer.hunks();
        assert_eq!(
            layer.run(b"@@ -1 +1 @@\n-old\n+new\n", *exit),
            RootEvent::Render(Event::Failed {
                request: REQUEST,
                class: crate::render::vocabulary::Class::Operational,
                at: None,
                failure: crate::render::vocabulary::Failure::Tool(crate::render::errors::Tool {
                    command: crate::render::errors::Command::Diff,
                    ended: ended.clone(),
                    tail: Box::new([])
                })
            }),
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

#[test]
fn git_s_failure_answers_its_exit_and_standard_error_s_tail() {
    let mut layer = Layer::new();
    layer.list();
    assert_eq!(
        layer.run_stderr(b"", b"fatal: not a repository\n", Exit::Code(128)),
        RootEvent::Render(Event::Failed {
            request: REQUEST,
            class: crate::render::vocabulary::Class::Operational,
            at: None,
            failure: crate::render::vocabulary::Failure::Tool(crate::render::errors::Tool {
                command: crate::render::errors::Command::RevParse,
                ended: crate::render::errors::Ended::Exit(128),
                tail: Box::new([bytes::copy_of(b"fatal: not a repository")])
            })
        })
    );
}
#[test]
fn a_run_that_exits_zero_holds_no_tail() {
    let mut layer = Layer::new();
    let Asked::Io(io::Request::Spawn { owner, .. }) = layer.resolve() else { panic!("listing spawn") };
    layer.spawned(owner, false, Exit::Code(0));
    assert_eq!(
        layer.run_stderr(b"", b"warning discarded\n", Exit::Code(0)),
        RootEvent::Render(Event::Listed { request: REQUEST, commit: super::fixtures::oid(), entries: Box::new([]) })
    );
}

#[test]
fn a_listing_past_its_entries_fails_as_configuration_at_its_path() {
    let mut layer = Layer::new();
    let Asked::Io(io::Request::Spawn { owner, .. }) = layer.resolve() else { panic!("listing spawn") };
    layer.spawned(owner, false, Exit::Code(0));
    layer.listing_error(crate::git_machine::listing::Error::TooMany { path: bytes::copy_of(b"x") });
    assert_eq!(
        layer.run(b"", Exit::Code(0)),
        RootEvent::Render(Event::Failed {
            request: REQUEST,
            class: crate::render::vocabulary::Class::Configuration,
            at: Some(bytes::copy_of(b"x")),
            failure: crate::render::vocabulary::Failure::TooLarge {
                limit: crate::render::errors::Limit::Entries,
                max: 65_536
            }
        })
    );
}
#[test]
fn a_4_097_byte_path_fails_as_configuration_at_its_prefix() {
    let mut layer = Layer::new();
    let Asked::Io(io::Request::Spawn { owner, .. }) = layer.resolve() else { panic!("listing spawn") };
    layer.spawned(owner, false, Exit::Code(0));
    assert_eq!(
        layer.run(&super::fixtures::listing(&[b'x'; 4_097], 1), Exit::Code(0)),
        RootEvent::Render(Event::Failed {
            request: REQUEST,
            class: crate::render::vocabulary::Class::Configuration,
            at: Some(bytes::copy_of(&[b'x'; 4_096])),
            failure: crate::render::vocabulary::Failure::TooLarge {
                limit: crate::render::errors::Limit::Path,
                max: 4_096
            }
        })
    );
}
#[test]
fn paths_past_the_total_fail_as_configuration_at_the_last_path() {
    let limits = crate::limits::Limits {
        git: crate::git_machine::limits::Limits { paths_total: 64, ..crate::git_machine::limits::SHIPPED },
        ..crate::limits::SHIPPED
    };
    let mut layer = Layer::with_form(crate::Form::Text, limits);
    let Asked::Io(io::Request::Spawn { owner, .. }) = layer.resolve() else { panic!("listing spawn") };
    layer.spawned(owner, false, Exit::Code(0));
    assert_eq!(
        layer.run(&super::fixtures::listing(b"a-path-that-is-thirty-bytes-long", 3), Exit::Code(0)),
        RootEvent::Render(Event::Failed {
            request: REQUEST,
            class: crate::render::vocabulary::Class::Configuration,
            at: Some(bytes::copy_of(b"a-path-that-is-thirty-bytes-long")),
            failure: crate::render::vocabulary::Failure::TooLarge {
                limit: crate::render::errors::Limit::Paths,
                max: 64
            }
        })
    );
}

#[test]
fn a_git_exit_waits_for_the_stdout_demand_already_in_flight() {
    use super::fixtures::{ERROR, OUTPUT};
    use skein_lib::stream::{Down, Read, Up};
    let mut layer = Layer::new();
    layer.list();
    let Some(Asked::Io(io::Request::Spawn { owner, .. })) = layer.asked() else { panic!("spawn") };
    layer.spawn_pending(owner, false);
    assert_eq!(
        layer.asked(),
        Some(Asked::Io(io::Request::Stream {
            stream: OUTPUT,
            down: Down::Demand { read: Read::Arrived { max: 66 }, room: 0 }
        }))
    );
    layer.up(io::Event::Exited { owner, exit: Exit::Code(128) });
    assert_eq!(
        layer.asked(),
        Some(Asked::Io(io::Request::Stream {
            stream: ERROR,
            down: Down::Demand { read: Read::Arrived { max: 16384 }, room: 0 }
        }))
    );
    layer.up(io::Event::Stream { owner: ERROR, up: Up::End });
    assert_eq!(layer.asked(), None, "the existing stdout demand owns its completion");
}
