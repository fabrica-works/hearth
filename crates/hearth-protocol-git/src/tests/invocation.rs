use crate::{
    invocation::{self, Command, Exit, Refused},
    oid::Oid,
};
const ID: &[u8] = b"ce013625030ba8dba906f756967f9e9ca394464a";
#[test]
fn each_command_s_vector_is_render_md_2_2_s_word_for_word() {
    let revision = invocation::revision(b"refs/heads/a").expect("valid ref");
    let commit = Oid::parse(ID).expect("valid id");
    let before = [0_u8; 32];
    let after = [0xab_u8; 32];
    let cases: [(Command<'_>, &[&[u8]]); 4] = [
        (
            Command::RevParse { repository: b"/a b/.git", revision: &revision },
            &[b"--git-dir=/a b/.git", b"rev-parse", b"--verify", b"--quiet", b"refs/heads/a^{commit}"],
        ),
        (
            Command::LsTree { repository: b"/repo", commit },
            &[b"--git-dir=/repo", b"ls-tree", b"-r", b"-l", b"-z", b"--full-tree", ID],
        ),
        (Command::CatFile { repository: b"/repo" }, &[b"--git-dir=/repo", b"cat-file", b"--batch"]),
        (
            Command::Diff { store: b"/a b", before: &before, after: &after },
            &[
                b"--git-dir=/dev/null",
                b"diff",
                b"--no-index",
                b"--no-color",
                b"--no-ext-diff",
                b"--no-textconv",
                b"--diff-algorithm=myers",
                b"--unified=3",
                b"--",
                b"/a b/cache/0000000000000000000000000000000000000000000000000000000000000000",
                b"/a b/cache/abababababababababababababababababababababababababababababababab",
            ],
        ),
    ];
    assert_eq!(invocation::GIT, b"/usr/bin/git", "fixed executable");
    for (command, expected) in cases {
        let arguments = invocation::arguments(command);
        assert_eq!(arguments.len(), expected.len(), "argument count");
        for (got, want) in arguments.iter().zip(expected) {
            assert_eq!(&**got, *want, "one argument");
        }
        for exit in [Exit::Code(0), Exit::Code(1), Exit::Code(2), Exit::Signal(9)] {
            let want = match command {
                Command::RevParse { .. } => exit == Exit::Code(0) || exit == Exit::Code(1),
                Command::LsTree { .. } | Command::CatFile { .. } => exit == Exit::Code(0),
                Command::Diff { .. } => exit == Exit::Code(1),
            };
            assert_eq!(invocation::succeeded(command, exit), want, "command exit");
        }
    }
}
#[test]
fn the_environment_is_given_whole_with_home_dev_null() {
    const EXPECTED: [&[u8]; 6] = [
        b"LC_ALL=C",
        b"GIT_CONFIG_NOSYSTEM=1",
        b"GIT_CONFIG_GLOBAL=/dev/null",
        b"GIT_NO_REPLACE_OBJECTS=1",
        b"GIT_TERMINAL_PROMPT=0",
        b"HOME=/dev/null",
    ];
    let env = invocation::environment();
    assert_eq!(env.len(), EXPECTED.len(), "complete environment");
    for (got, expected) in env.iter().zip(EXPECTED) {
        assert_eq!(&**got, expected, "ordered variable");
    }
}
#[test]
fn a_ref_name_of_the_alphabet_up_to_255_bytes_passes() {
    const CASES: [&[u8]; 7] = [b"refs/heads/a-b_c.d/9", b"ABCDEF123", b"ce01362", b"a..b", b"/", b".", &[b'x'; 255]];
    for input in CASES {
        assert_eq!(invocation::revision(input).expect("valid alphabet").bytes(), input, "guard preserves bytes");
    }
}
#[test]
fn a_byte_outside_the_alphabet_or_a_256th_byte_is_refused() {
    const CASES: [(&[u8], Refused); 6] = [
        (b"", Refused::Empty),
        (&[b'x'; 256], Refused::Long),
        (b"a b", Refused::Byte { at: 1 }),
        (b"a\0", Refused::Byte { at: 1 }),
        (b"a^{commit}", Refused::Byte { at: 1 }),
        (b"a\xff", Refused::Byte { at: 1 }),
    ];
    for (input, expected) in CASES {
        assert_eq!(invocation::revision(input), Err(expected), "refused {input:?}");
    }
}
#[test]
fn a_revision_starting_with_a_dash_never_reaches_git() {
    for input in [b"-".as_slice(), b"--help", b"-a"] {
        assert_eq!(invocation::revision(input), Err(Refused::Dash), "leading dash");
    }
}
