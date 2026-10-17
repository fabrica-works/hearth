mod fixtures;
use crate::{
    Call, Form, argv,
    records::startup::{Misuse, Opened, Startup},
};
use fixtures::words;
use std::{ffi::OsStr, path::Path};

#[test]
fn render_s_options_become_the_call_and_its_form() {
    let invocation = argv::invocation(
        &words(&["render", "x1-prod", "x2-prod", "--at", "main", "--repo", "/", "--store", "/store", "--json"]),
        None,
        None,
        Path::new("/"),
    )
    .expect("invocation");
    assert_eq!(
        invocation.call,
        Call::Render {
            repository: b"/".as_slice().into(),
            revision: b"main".as_slice().into(),
            targets: Box::new([b"x1-prod".as_slice().into(), b"x2-prod".as_slice().into()])
        },
        "options and targets"
    );
    assert_eq!(invocation.form, Form::JsonLines, "json form");
    assert_eq!(invocation.store, Path::new("/store"), "explicit store");
}
const MISUSES: &[(&[&str], Misuse, &[u8])] = &[
    (&["bogus"], Misuse::Command, b"bogus"),
    (&["diff"], Misuse::Command, b"diff"),
    (&["render", "--resolve"], Misuse::Option, b"--resolve"),
    (&["render", "--bogus"], Misuse::Option, b"--bogus"),
    (&["render", "--at"], Misuse::Value, b"--at"),
];
#[test]
fn an_unknown_command_or_option_is_usage() {
    for &(args, misuse, word) in MISUSES {
        assert_eq!(
            argv::invocation(&words(args), None, None, Path::new("/")),
            Err(Startup::Usage { misuse, word: word.into() }),
            "{args:?}"
        );
    }
}
const JSON_ARGS: &[&[&str]] = &[&["render", "--json", "--bogus"], &["render", "--bogus", "--json"]];
#[test]
fn json_anywhere_in_argv_gives_the_refusal_its_form() {
    for args in JSON_ARGS {
        assert_eq!(argv::form(&words(args)), Form::JsonLines, "{args:?}");
    }
}
#[test]
fn a_malformed_revision_is_usage_before_the_loop() {
    assert_eq!(
        argv::invocation(&words(&["render", "--at", "-x"]), None, None, Path::new("/")),
        Err(Startup::Usage { misuse: Misuse::Revision, word: b"-x".as_slice().into() }),
        "revision guard"
    );
}
const STORES: &[(Option<&str>, Option<&str>, &str)] =
    &[(Some("/xdg"), Some("/home"), "/xdg/hearth/store"), (None, Some("/home"), "/home/.cache/hearth/store")];
#[test]
fn the_store_is_xdg_cache_home_s_then_home_s_cache() {
    for &(xdg, home, store) in STORES {
        let invocation = argv::invocation(
            &words(&["render", "--repo", "/"]),
            xdg.map(OsStr::new),
            home.map(OsStr::new),
            Path::new("/"),
        )
        .expect("default store");
        assert_eq!(invocation.store, Path::new(store), "{xdg:?} {home:?}");
    }
}
const UNSET: &[(&str, &str)] = &[("", "/home"), ("relative", "/home")];
#[test]
fn an_empty_or_relative_variable_counts_as_unset() {
    for &(xdg, home) in UNSET {
        let invocation = argv::invocation(
            &words(&["render", "--repo", "/"]),
            Some(OsStr::new(xdg)),
            Some(OsStr::new(home)),
            Path::new("/"),
        )
        .expect("home fallback");
        assert_eq!(invocation.store, Path::new("/home/.cache/hearth/store"), "{xdg:?}");
    }
}
const NO_STORE: &[(Option<&str>, Option<&str>)] =
    &[(None, None), (Some(""), Some("relative")), (Some("relative"), Some(""))];
#[test]
fn with_neither_variable_the_store_is_unopened() {
    for &(xdg, home) in NO_STORE {
        assert_eq!(
            argv::invocation(&words(&["render"]), xdg.map(OsStr::new), home.map(OsStr::new), Path::new("/")),
            Err(Startup::Unopened {
                what: Opened::Store,
                path: None,
                error: b"neither XDG_CACHE_HOME nor HOME is absolute".as_slice().into()
            }),
            "{xdg:?} {home:?}"
        );
    }
}
#[test]
fn a_work_tree_s_repository_is_its_dot_git() {
    use std::os::unix::ffi::OsStrExt;
    let scratch = skein_scratch::Scratch::new("argv-repository");
    std::fs::create_dir(scratch.path().join(".git")).expect("git directory");
    let invocation = argv::invocation(&words(&["render", "--store", "/store"]), None, None, scratch.path())
        .expect("repository found");
    let Call::Render { repository, .. } = invocation.call else { panic!("render call") };
    assert_eq!(
        Path::new(OsStr::from_bytes(&repository)).strip_prefix(scratch.path()).expect("inside scratch"),
        Path::new(".git"),
        "git directory"
    );
}
#[test]
fn two_seeds_drawn_at_startup_differ() {
    assert_ne!(crate::seed().expect("seed"), crate::seed().expect("another seed"), "fresh generator seeds");
}

#[test]
fn the_revision_defaults_to_head() {
    let invocation =
        argv::invocation(&words(&["render", "--repo", "/", "--store", "/store"]), None, None, Path::new("/"))
            .expect("invocation");
    let Call::Render { revision, .. } = invocation.call else { panic!("render call") };
    assert_eq!(&*revision, b"HEAD", "default revision");
}
