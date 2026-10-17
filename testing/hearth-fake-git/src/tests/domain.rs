use super::fixtures;
use crate::{
    diff::{self, Diff, Hunk, Line, Span},
    objects::{Mode, Oid, Tag, Type},
    repository::{Repositories, Repository, Resolved},
};

type CommitCase = (&'static str, fn() -> crate::objects::Commit, &'static [u8; 40]);
const COMMITS: &[CommitCase] = &[
    ("hello", fixtures::hello, b"663d5cdec21ff28ca3e39f3f92e67bbd9b5d72f6"),
    ("mixed", fixtures::mixed, b"739a369fdfee74b857461469d8a3a358285a3579"),
];
#[test]
fn a_commit_the_world_lays_has_the_id_real_git_gives_it() {
    let mut repository = Repository::new(b"refs/heads/main");
    let (case, build, expected) = COMMITS[0];
    assert_eq!(&repository.commit(&build()).hex(), expected, "{case}");
}
#[test]
fn files_a_link_and_a_submodule_give_git_s_commit_id() {
    let mut repository = Repository::new(b"refs/heads/main");
    let (case, build, expected) = COMMITS[1];
    assert_eq!(&repository.commit(&build()).hex(), expected, "{case}");
}
const BLOB: &[(&[u8; 40], Mode, Option<u64>)] = &[(b"ce013625030ba8dba906f756967f9e9ca394464a", Mode::File, Some(6))];
#[test]
fn the_listing_names_the_blob_as_real_git_does() {
    let mut repository = Repository::new(b"refs/heads/main");
    let commit = repository.commit(&fixtures::hello());
    let listed = repository.list(commit).expect("stored commit lists");
    assert_eq!((&listed[0].oid.hex(), listed[0].mode, listed[0].size), BLOB[0]);
}
const RESOLVED: &[(&[u8], Resolved)] = &[
    (
        b"main",
        Resolved::Commit(Oid([
            0x66, 0x3d, 0x5c, 0xde, 0xc2, 0x1f, 0xf2, 0x8c, 0xa3, 0xe3, 0x9f, 0x3f, 0x92, 0xe6, 0x7b, 0xbd, 0x9b, 0x5d,
            0x72, 0xf6,
        ])),
    ),
    (
        b"release",
        Resolved::Commit(Oid([
            0x66, 0x3d, 0x5c, 0xde, 0xc2, 0x1f, 0xf2, 0x8c, 0xa3, 0xe3, 0x9f, 0x3f, 0x92, 0xe6, 0x7b, 0xbd, 0x9b, 0x5d,
            0x72, 0xf6,
        ])),
    ),
    (
        b"663d5cd",
        Resolved::Commit(Oid([
            0x66, 0x3d, 0x5c, 0xde, 0xc2, 0x1f, 0xf2, 0x8c, 0xa3, 0xe3, 0x9f, 0x3f, 0x92, 0xe6, 0x7b, 0xbd, 0x9b, 0x5d,
            0x72, 0xf6,
        ])),
    ),
    (b"absent", Resolved::Missing),
];
#[test]
fn a_tag_and_a_branch_of_one_name_resolve_to_the_tag() {
    let mut repository = Repository::new(b"refs/heads/main");
    let hello = repository.commit(&fixtures::hello());
    let mixed = repository.commit(&fixtures::mixed());
    repository.set_ref(b"refs/heads/main", mixed);
    repository.set_ref(b"refs/tags/main", hello);
    assert_eq!(repository.resolve(RESOLVED[0].0), RESOLVED[0].1);
}
#[test]
fn an_annotated_tag_peels_to_its_commit() {
    let mut repository = Repository::new(b"refs/heads/main");
    let commit = repository.commit(&fixtures::hello());
    repository.tag(&Tag {
        name: b"release".as_slice().into(),
        commit,
        tagger: fixtures::hearth(),
        message: b"release\n".as_slice().into(),
    });
    assert_eq!(repository.resolve(RESOLVED[1].0), RESOLVED[1].1);
}
const TREE: &[Resolved] = &[Resolved::NotACommit { kind: Type::Tree }];
#[test]
fn a_tree_s_id_resolves_to_no_commit_naming_the_tree() {
    let mut repository = Repository::new(b"refs/heads/main");
    let commit = repository.commit(&fixtures::hello());
    let (_, body) = repository.object(commit).expect("stored commit");
    let tree = body[5..45].to_vec();
    assert_eq!(repository.resolve(&tree), TREE[0]);
}
#[test]
fn a_unique_seven_digit_prefix_resolves_its_commit() {
    let mut repository = Repository::new(b"refs/heads/main");
    repository.commit(&fixtures::hello());
    assert_eq!(repository.resolve(RESOLVED[2].0), RESOLVED[2].1);
}
type HunkCase = (&'static [u8], &'static [u8], Span, Span, &'static [u8], &'static [(Line, &'static [u8])]);
const CHANGED: &[HunkCase] = &[(
    b"a\nb\nc\nd\ne\nf\ng\nh\n",
    b"a\nb\nc\nd\nE\nf\ng\nh\n",
    Span { start: 2, count: 7 },
    Span { start: 2, count: 7 },
    b"a",
    &[
        (Line::Context, b"b\n"),
        (Line::Context, b"c\n"),
        (Line::Context, b"d\n"),
        (Line::Removed, b"e\n"),
        (Line::Added, b"E\n"),
        (Line::Context, b"f\n"),
        (Line::Context, b"g\n"),
        (Line::Context, b"h\n"),
    ],
)];
#[test]
fn one_changed_line_gives_one_hunk_with_three_lines_of_context() {
    for &(before, after, left, right, function, lines) in CHANGED {
        let expected = Diff::Hunks(Box::new([Hunk {
            before: left,
            after: right,
            function: function.into(),
            lines: lines.iter().map(|(kind, bytes)| (*kind, (*bytes).into())).collect(),
        }]));
        assert_eq!(diff::lines(before, after), expected);
    }
}
const SEPARATIONS: &[(usize, usize)] = &[(9, 1), (10, 2)];
#[test]
fn changes_six_lines_apart_share_a_hunk_and_seven_do_not() {
    let before: Vec<_> = (1..=20).flat_map(|n| format!("line {n}\n").into_bytes()).collect();
    for &(second, expected) in SEPARATIONS {
        let after: Vec<_> = (1..=20)
            .flat_map(|n| format!("{} {n}\n", if n == 2 || n == second { "changed" } else { "line" }).into_bytes())
            .collect();
        let Diff::Hunks(hunks) = diff::lines(&before, &after) else {
            panic!("text hunks");
        };
        assert_eq!(hunks.len(), expected, "second change at {second}");
    }
}
const BINARY: &[(&[u8], &[u8], Diff)] = &[(b"a\0", b"a", Diff::Binary), (b"a", b"a\0", Diff::Binary)];
#[test]
fn a_nul_in_the_first_8000_bytes_makes_the_pair_binary() {
    for (before, after, expected) in BINARY {
        assert_eq!(&diff::lines(before, after), expected);
    }
}
type LaidCase = (&'static [u8], u32, u32, u32, Option<&'static [u8]>);
const LAID: &[LaidCase] = &[
    (b"/srv", 0o755, 0, 0, None),
    (b"/srv/hello.git", 0o755, 1000, 1000, None),
    (b"/srv/hello.git/HEAD", 0o644, 1000, 1000, Some(b"ref: refs/heads/main\n")),
];
#[test]
fn laying_a_repository_makes_only_its_directory_and_head() {
    use hearth_fake_machine::linux::{Ids, Machine, Sizes};
    let mut machine = Machine::new(1, Sizes::tiny());
    let mut repositories = Repositories::default();
    repositories.lay(
        &mut machine,
        b"/srv/hello.git",
        Ids { user: 1000, group: 1000 },
        Repository::new(b"refs/heads/main"),
    );
    for &(path, mode, user, group, bytes) in LAID {
        let facts = machine.facts(path).expect("laid path");
        assert_eq!((facts.mode, facts.user, facts.group), (mode, user, group));
        if let Some(bytes) = bytes {
            assert_eq!(machine.contents(path), Some(bytes));
        }
    }
}
