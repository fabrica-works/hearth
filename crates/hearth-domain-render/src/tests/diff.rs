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
            } => (path.as_ref(), Some(*before), Some(*after)),
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
