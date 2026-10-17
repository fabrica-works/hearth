use super::fixtures;
use crate::{
    diff::{self, Layer, Made, Needs, Rebuild, Side, TargetChange},
    parts::config::{self, EntryChange, Node},
    vocabulary::Class,
};
use hearth_lib::encode::Digest;
use skein_lib::bytes;
const REBUILDS: &[(&[Layer], Rebuild)] = &[
    (&[], Rebuild::Nothing),
    (&[Layer::Other], Rebuild::Nothing),
    (&[Layer::Slot, Layer::Other], Rebuild::SlotOnly),
    (&[Layer::Software, Layer::Slot], Rebuild::FromSoftware),
    (&[Layer::Config, Layer::Slot], Rebuild::FromConfig),
    (&[Layer::Config, Layer::Packages], Rebuild::FromPackages { resolution: Needs::Needed }),
];
#[test]
fn the_lowest_changed_layer_is_the_rebuild() {
    for (changed, expected) in REBUILDS {
        assert_eq!(diff::rebuild(changed, Needs::Needed), *expected, "render.md 6.2: {changed:?}");
    }
}
#[test]
fn equal_spec_digests_are_same_whatever_the_commits() {
    let before = fixtures::spec(b"x1-prod", &[]);
    let mut after = before.clone();
    after.commit = crate::revision::Oid(bytes::copy_of(b"abcdef0123456789abcdef0123456789abcdef01"));
    assert_eq!(
        diff::compare(Some(&Made::Spec(Box::new(before))), Some(&Made::Spec(Box::new(after))), Needs::Needed),
        TargetChange::Same,
        "commit outside digest"
    );
}
const ONE_SIDE: &[(bool, TargetChange)] =
    &[(true, TargetChange::Removed), (false, TargetChange::Added { spec_digest: Digest([2; 32]) })];
#[test]
fn a_target_on_one_side_alone_is_added_or_removed() {
    let made = Made::Spec(Box::new(fixtures::spec(b"x1-prod", &[])));
    for (from, expected) in ONE_SIDE {
        let pair = if *from { (Some(&made), None) } else { (None, Some(&made)) };
        assert_eq!(diff::compare(pair.0, pair.1, Needs::Needed), *expected, "from={from}");
    }
}
const FAILED_SIDES: &[(bool, TargetChange)] =
    &[(true, TargetChange::Failed { side: Side::From }), (false, TargetChange::Failed { side: Side::To })];
#[test]
fn a_target_one_side_has_and_failed_there_is_failed_not_added() {
    const FAILED: Made = Made::Failed(Class::Configuration);
    for (from, expected) in FAILED_SIDES {
        let pair = if *from { (Some(&FAILED), None) } else { (None, Some(&FAILED)) };
        assert_eq!(diff::compare(pair.0, pair.1, Needs::Needed), *expected, "from={from}");
    }
}
#[test]
fn a_target_failing_on_both_sides_is_failed_both_with_each_sides_errors() {
    const FROM: Made = Made::Failed(Class::Configuration);
    const TO: Made = Made::Failed(Class::Operational);
    assert_eq!(
        diff::compare(Some(&FROM), Some(&TO), Needs::Needed),
        TargetChange::Failed { side: Side::Both },
        "neither side rendered"
    );
}
const TREE: &[(&[u8], Option<Digest>, Option<Digest>)] = &[
    (b"/added", None, Some(Digest([2; 32]))),
    (b"/changed", Some(Digest([1; 32])), Some(Digest([2; 32]))),
    (b"/removed", Some(Digest([1; 32])), None),
];
#[test]
fn tree_entries_are_added_removed_or_changed_by_path() {
    let (before, after) = fixtures::diff_trees();
    let change = config::change(Some(&before), Some(&after));
    assert_eq!(change.tree.len(), 3, "three changed paths");
    for (entry, expected) in change.tree.iter().zip(TREE) {
        let (path, before, after) = match entry {
            EntryChange::Added { path, node: Node::File { content_digest, .. } } => {
                (path.as_ref(), None, Some(*content_digest))
            }
            EntryChange::Removed { path, node: Node::File { content_digest, .. } } => {
                (path.as_ref(), Some(*content_digest), None)
            }
            EntryChange::Changed {
                path,
                before: Node::File { content_digest: before, .. },
                after: Node::File { content_digest: after, .. },
                ..
            } => (path.as_ref(), Some(*before), Some(*after)),
            EntryChange::Added { .. } | EntryChange::Removed { .. } | EntryChange::Changed { .. } => {
                unreachable!("file fixtures")
            }
        };
        assert_eq!((path, before, after), *expected, "byte path order: {entry:?}");
    }
}
#[test]
fn changed_parts_are_compared_by_digest_and_rebuild_from_config() {
    let (before, after) = fixtures::changed_specs();
    assert_eq!(
        diff::compare(Some(&Made::Spec(Box::new(before))), Some(&Made::Spec(Box::new(after))), Needs::Needed),
        TargetChange::Changed {
            rebuild: Rebuild::FromConfig,
            parts: Box::new([diff::PartChange::Config(config::Change {
                tree: Box::new([EntryChange::Added {
                    path: bytes::copy_of(b"/added"),
                    node: Node::File {
                        content_digest: Digest([3; 32]),
                        mode: 0o644,
                        owner: bytes::copy_of(b"root"),
                        group: bytes::copy_of(b"root")
                    }
                }])
            })])
        },
        "config digest changed"
    );
}
#[test]
fn the_sides_render_in_turn_from_then_to() {
    use crate::vocabulary::{Event, Output, Request};
    use skein_lib::Token;
    let (mut render, env, mut out) = fixtures::diff_call();
    assert_eq!(
        out.pop(),
        Some(Output::Request(Request::List {
            request: Token::new(1),
            repository: bytes::copy_of(b"/repo"),
            revision: bytes::copy_of(b"before")
        })),
        "from requested first"
    );
    assert!(!render.work_pending(), "to waits for from's answer");
    crate::step(
        &mut render,
        &env,
        Event::Listed {
            request: Token::new(1),
            commit: crate::revision::Oid(bytes::copy_of(b"a")),
            entries: Box::new([]),
        },
        &mut out,
    );
    crate::resume(&mut render, &env, &mut out);
    assert_eq!(
        out.pop(),
        Some(Output::Request(Request::List {
            request: Token::new(1),
            repository: bytes::copy_of(b"/repo"),
            revision: bytes::copy_of(b"after")
        })),
        "to follows from's last answer"
    );
}

struct FilePair {
    before: u64,
    after: u64,
    digest: Digest,
    expected: Option<(Digest, Digest)>,
}
const FILE_PAIRS: &[FilePair] = &[
    FilePair { before: 9, after: 22, digest: Digest([2; 32]), expected: Some((Digest([1; 32]), Digest([2; 32]))) },
    FilePair {
        before: 1_048_575,
        after: 22,
        digest: Digest([2; 32]),
        expected: Some((Digest([1; 32]), Digest([2; 32]))),
    },
    FilePair { before: 1_048_576, after: 22, digest: Digest([2; 32]), expected: None },
    FilePair { before: 9, after: 1_048_576, digest: Digest([2; 32]), expected: None },
    FilePair { before: 9, after: 9, digest: Digest([1; 32]), expected: None },
];
#[test]
fn hunks_are_asked_only_for_two_files_under_1_mib_whose_digests_differ() {
    for FilePair { before, after, digest, expected } in FILE_PAIRS {
        let (mut entry, sizes_before, sizes_after) = fixtures::changed_file(*before, *after, *digest);
        assert_eq!(
            entry.pair(&sizes_before, &sizes_after, crate::limits::SHIPPED.hunk_file),
            *expected,
            "{before}/{after}/{digest:?}"
        );
    }
}
#[test]
fn two_changed_files_keep_their_cached_sizes() {
    let (mut entry, before, after) = fixtures::changed_file(9, 22, Digest([2; 32]));
    entry.pair(&before, &after, crate::limits::SHIPPED.hunk_file);
    let EntryChange::Changed { sizes, .. } = entry else { panic!("changed file") };
    assert_eq!(sizes, Some(config::Sizes { before: 9, after: 22 }));
}
#[test]
fn added_or_removed_files_never_request_hunks() {
    let (before, after) = fixtures::diff_trees();
    let mut change = config::change(Some(&before), Some(&after));
    let sizes = crate::call::Sizes::with_capacity(0);
    for entry in &mut change.tree {
        assert_eq!(entry.pair(&sizes, &sizes, crate::limits::SHIPPED.hunk_file), None, "no pair of cached files");
    }
}
#[test]
fn a_failed_hunks_request_leaves_the_file_bare_and_the_call_try_again() {
    use crate::vocabulary::{Answer, Output};
    let (mut render, env, mut out) = fixtures::hunk_call();
    crate::step(&mut render, &env, fixtures::hunk_failure(), &mut out);
    crate::resume(&mut render, &env, &mut out);
    let Some(Output::Answer(Answer::Change(change))) = out.pop() else { panic!("change stands") };
    let mut target_change = change.change;
    let tree = diff::tree(&mut target_change);
    assert_eq!(
        tree,
        &[EntryChange::Changed {
            path: bytes::copy_of(b"/etc/motd"),
            before: Node::File {
                content_digest: Digest([1; 32]),
                mode: 0o644,
                owner: bytes::copy_of(b"root"),
                group: bytes::copy_of(b"root")
            },
            after: Node::File {
                content_digest: Digest([2; 32]),
                mode: 0o755,
                owner: bytes::copy_of(b"root"),
                group: bytes::copy_of(b"root")
            },
            sizes: Some(config::Sizes { before: 9, after: 22 }),
            hunks: None,
        }]
    );
}
#[test]
fn a_failed_hunks_request_ends_the_call_try_again() {
    use crate::vocabulary::{Answer, Outcome, Output};
    let (mut render, env, mut out) = fixtures::hunk_call();
    crate::step(&mut render, &env, fixtures::hunk_failure(), &mut out);
    crate::resume(&mut render, &env, &mut out);
    drop(out.pop());
    crate::resume(&mut render, &env, &mut out);
    drop(out.pop());
    crate::resume(&mut render, &env, &mut out);
    let Some(Output::Answer(Answer::Rendered { outcome, .. })) = out.pop() else { panic!("terminal") };
    assert_eq!(outcome, Outcome::TryAgain);
}

#[test]
fn a_meta_change_holds_each_moved_field_before_and_after_and_no_other() {
    use crate::parts::meta::{self, NamePair};
    let before = fixtures::meta();
    let mut after = before.clone();
    after.base = bytes::copy_of(b"x2");
    after.environment = bytes::copy_of(b"lab");
    assert_eq!(
        meta::change(&before, &after),
        Some(meta::Change {
            base: Some(NamePair { before: bytes::copy_of(b"x1"), after: bytes::copy_of(b"x2") }),
            member: None,
            environment: Some(NamePair { before: bytes::copy_of(b"prod"), after: bytes::copy_of(b"lab") }),
            placement: None,
        })
    );
}
#[test]
fn a_member_changed_alone_is_a_meta_change_rebuilding_nothing() {
    use crate::parts::meta::{Change, MemberPair};
    let before = fixtures::spec(b"x1-prod", &[]);
    let mut after = before.clone();
    after.digest = Digest([3; 32]);
    after.meta.member = Some(bytes::copy_of(b"a"));
    assert_eq!(
        diff::compare(Some(&Made::Spec(Box::new(before))), Some(&Made::Spec(Box::new(after))), Needs::Needed),
        TargetChange::Changed {
            parts: Box::new([diff::PartChange::Meta(Change {
                base: None,
                member: Some(MemberPair { before: None, after: Some(bytes::copy_of(b"a")) }),
                environment: None,
                placement: None,
            })]),
            rebuild: Rebuild::Nothing
        }
    );
}
#[test]
fn each_error_of_a_diff_names_its_side() {
    use crate::vocabulary::Answer;
    let answers = fixtures::failed_diff(256, 1);
    let mut sides = skein_lib::List::with_capacity(2);
    for answer in &answers {
        if let Answer::Error(error) = answer {
            sides.push(error.side).expect("two sides");
        }
    }
    assert_eq!(sides.as_slice(), &[Some(Side::From), Some(Side::To)]);
}
#[test]
fn the_diffs_errors_are_bounded_once_over_both_sides() {
    use crate::vocabulary::Answer;
    let answers = fixtures::failed_diff(256, 130);
    let mut errors = 0_u32;
    for answer in &answers {
        if let Answer::Error(_) = answer {
            errors += 1;
        }
    }
    assert_eq!(errors, 256);
    let Some(Answer::Rendered { left_out, .. }) = answers.last() else { panic!("terminal") };
    assert_eq!(*left_out, 4);
}

#[test]
fn a_target_failing_on_both_sides_has_each_sides_errors_with_its_side() {
    use crate::vocabulary::Answer;
    let answers = fixtures::failed_diff(256, 1);
    assert_eq!(
        answers.first(),
        Some(&Answer::Change(diff::Change {
            target: bytes::copy_of(b"x1-prod"),
            change: TargetChange::Failed { side: Side::Both },
        }))
    );
}
#[test]
fn a_failed_hunks_request_leaves_its_file_bare_then_a_tool_error() {
    use crate::{
        errors::{Command, Ended, Error, Kind, Tool},
        vocabulary::{Answer, Class},
    };
    let (mut render, env, mut out) = fixtures::hunk_call();
    crate::step(&mut render, &env, fixtures::hunk_failure(), &mut out);
    let answers = fixtures::finish_render(&mut render, &env, &mut out);
    let mut error = None;
    for answer in &answers {
        if let Answer::Error(_) = answer {
            error = Some(answer);
        }
    }
    assert_eq!(
        error,
        Some(&Answer::Error(Error {
            side: Some(Side::To),
            kind: Kind::Tool(Tool { command: Command::Diff, ended: Ended::Exit(2), tail: Box::new([]) }),
            class: Class::Operational,
            at: None,
            also: None,
            target: Some(bytes::copy_of(b"x1-prod")),
            task: None,
        }))
    );
}

#[test]
fn omitted_diff_errors_still_decide_its_outcome() {
    use crate::vocabulary::{Answer, Outcome};
    let answers = fixtures::failed_diff(0, 1);
    let Some(Answer::Rendered { outcome, .. }) = answers.last() else { panic!("terminal") };
    assert_eq!(*outcome, Outcome::Failed);
}
#[test]
fn a_diff_whose_listing_fails_keeps_its_error() {
    use crate::{
        errors::{Command, Ended, Error, Kind, Tool},
        vocabulary::{Answer, Class, Event, Failure},
    };
    let (mut render, env, mut out) = fixtures::diff_call();
    drop(out.pop());
    crate::step(
        &mut render,
        &env,
        Event::Failed {
            request: skein_lib::Token::new(1),
            class: Class::Operational,
            at: None,
            failure: Failure::Tool(Tool { command: Command::RevParse, ended: Ended::Start, tail: Box::new([]) }),
        },
        &mut out,
    );
    let answers = fixtures::finish_render(&mut render, &env, &mut out);
    assert_eq!(
        answers.first(),
        Some(&Answer::Error(Error {
            side: Some(Side::From),
            kind: Kind::Tool(Tool { command: Command::RevParse, ended: Ended::Start, tail: Box::new([]) }),
            class: Class::Operational,
            at: None,
            also: None,
            target: None,
            task: None,
        }))
    );
}

mod exports;
mod packages;
