use super::fixtures;
use crate::{
    diff::*,
    form::{self, Form},
    limits,
    outcome::Outcome,
    records::{Record, Terminal, Type},
};
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
const CHANGED: [&str; 1] = [fixtures::WG0_TEXT];
#[test]
fn a_changed_file_prints_its_hunks_as_render_md_shows() {
    let printed = form::print(Form::Text, &Record::Change(fixtures::wg0_changed()), None, &limits::SHIPPED);
    assert_eq!(
        printed.as_deref(),
        Ok((*CHANGED.first().expect("one expected row")).as_bytes()),
        "render.md, 6.3, three lines of context"
    );
}
const NOTHING: [&str; 1] = ["ion-prod                                            nothing to build\n"];
#[test]
fn nothing_to_build_prints_at_the_rebuild_column() {
    let record = change(b"ion-prod", TargetChange::Changed { parts: Box::new([]), rebuild: Rebuild::Nothing });
    assert_eq!(text(&record).as_deref(), Ok((*NOTHING.first().expect("one expected row")).as_bytes()));
}
const SAME: [&[u8]; 1] = [b""];
#[test]
fn a_same_target_prints_no_line() {
    assert_eq!(
        text(&change(b"carbon-prod", TargetChange::Same)).as_deref(),
        Ok(*SAME.first().expect("one expected row"))
    );
}
const TALLIES: [(u32, u32, u32, &str); 4] = [
    (11, 2, 9, "\n2 of 11 targets change; 9 are unchanged.\n"),
    (1, 1, 0, "\n1 of 1 target changes; 0 are unchanged.\n"),
    (1, 0, 1, "\n0 of 1 target change; 1 is unchanged.\n"),
    (0, 0, 0, "\n0 of 0 targets change; 0 are unchanged.\n"),
];
#[test]
fn the_tally_ends_the_diff_after_a_blank_line() {
    for (targets, changed, unchanged, expected) in TALLIES {
        assert_eq!(
            text(&terminal(targets, changed, unchanged)).as_deref(),
            Ok(expected.as_bytes()),
            "{targets} targets, {changed} changes"
        );
    }
}
const NO_HUNKS: [&str; 1] = [concat!(
    "carbon-prod                                         rebuild from config\n",
    "  config   ~ /etc/systemd/network/wg0.netdev        0644 root:root\n",
)];
#[test]
fn a_binary_file_prints_no_hunk_line() {
    let mut value = fixtures::wg0_changed();
    hunks(&mut value).binary = true;
    assert_eq!(text(&Record::Change(value)).as_deref(), Ok((*NO_HUNKS.first().expect("one expected row")).as_bytes()));
}
const JSON_CHANGE: [&str; 1] = [concat!(
    r#"{"record":"change","target":"carbon-prod","kind":"changed","changed":{"parts":[{"kind":"config","tree":[{"after":{"content_digest":"2222222222222222222222222222222222222222222222222222222222222222","group":"root","kind":"file","mode":"0644","owner":"root"},"before":{"content_digest":"1111111111111111111111111111111111111111111111111111111111111111","group":"root","kind":"file","mode":"0644","owner":"root"},"hunks":{"binary":false,"left_out":0,"lines":[{"kind":"header","text":"@@ -9,5 +9,5 @@ ListenPort=51820"},{"kind":"context","text":"[WireGuardPeer]"},{"kind":"context","text":"PublicKey=m3NY8…="},{"kind":"context","text":"AllowedIPs=10.0.0.1/32"},{"kind":"removed","text":"Endpoint=194.164.162.157:51820"},{"kind":"added","text":"Endpoint=194.164.162.158:51820"},{"kind":"context","text":"PersistentKeepalive=25"}]},"kind":"changed","path":"/etc/systemd/network/wg0.netdev","sizes":{"after":268,"before":268}}]}],"rebuild":{"kind":"from_config"}}}"#,
    "\n",
)];
#[test]
fn json_gives_a_change_with_its_rebuild_and_parts() {
    let printed =
        form::print(Form::JsonLines, &Record::Change(fixtures::wg0_changed()), Some(Type::Change), &limits::SHIPPED);
    assert_eq!(printed.as_deref(), Ok((*JSON_CHANGE.first().expect("one expected row")).as_bytes()));
}
const JSON_TALLY: [&str; 1] = [concat!(
    r#"{"record":"outcome","outcome":"negative","from":"main","to":"HEAD","targets":11,"changed":2,"unchanged":9}"#,
    "\n"
)];
#[test]
fn json_gives_the_tally_in_the_outcome_record() {
    let printed = form::print(Form::JsonLines, &terminal(11, 2, 9), Some(Type::Change), &limits::SHIPPED);
    assert_eq!(printed.as_deref(), Ok((*JSON_TALLY.first().expect("one expected row")).as_bytes()));
}
const ADDED: [&str; 1] = ["carbon-prod                                         added spec 123456789abc\n"];
#[test]
fn an_added_target_prints_added_spec_and_its_twelve_digits() {
    let record = change(
        b"carbon-prod",
        TargetChange::Added { spec_digest: fixtures::digest([0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc]) },
    );
    assert_eq!(text(&record).as_deref(), Ok((*ADDED.first().expect("one expected row")).as_bytes()));
}
const REMOVED: [&str; 1] = ["carbon-prod                                         removed\n"];
#[test]
fn a_removed_target_prints_removed() {
    assert_eq!(
        text(&change(b"carbon-prod", TargetChange::Removed)).as_deref(),
        Ok((*REMOVED.first().expect("one expected row")).as_bytes())
    );
}
const FAILED: [(Side, &str); 3] = [
    (Side::From, "carbon-prod                                         failed (from)\n"),
    (Side::To, "carbon-prod                                         failed (to)\n"),
    (Side::Both, "carbon-prod                                         failed (both)\n"),
];
#[test]
fn a_failed_target_prints_failed_and_its_side() {
    for (side, expected) in FAILED {
        assert_eq!(
            text(&change(b"carbon-prod", TargetChange::Failed { side })).as_deref(),
            Ok(expected.as_bytes()),
            "{side:?}"
        );
    }
}
const FAILED_TALLY: [&str; 1] = ["\n0 of 1 target change; 0 are unchanged.\n"];
#[test]
fn a_failed_target_counts_among_the_targets_only() {
    assert_eq!(text(&terminal(1, 0, 0)).as_deref(), Ok((*FAILED_TALLY.first().expect("one expected row")).as_bytes()));
}
const LEFT_OUT: [&str; 1] = [concat!(
    "carbon-prod                                         rebuild from config\n",
    "  config   ~ /etc/systemd/network/wg0.netdev        0644 root:root\n",
    "               ... 5 lines left out\n",
)];
#[test]
fn hunk_lines_left_out_print_their_count() {
    let mut value = fixtures::wg0_changed();
    *hunks(&mut value) = Hunks { lines: Box::new([]), left_out: 5, binary: false };
    assert_eq!(text(&Record::Change(value)).as_deref(), Ok((*LEFT_OUT.first().expect("one expected row")).as_bytes()));
}
const REBUILDS: [(Rebuild, &str); 5] = [
    (
        Rebuild::FromPackages { resolution: Needs::Known },
        "carbon-prod                                         rebuild from packages\n",
    ),
    (
        Rebuild::FromPackages { resolution: Needs::Needed },
        "carbon-prod                                         rebuild from packages, resolution needed\n",
    ),
    (Rebuild::FromSoftware, "carbon-prod                                         rebuild from software\n"),
    (Rebuild::FromConfig, "carbon-prod                                         rebuild from config\n"),
    (Rebuild::SlotOnly, "carbon-prod                                         new slot only\n"),
];
#[test]
fn every_rebuild_prints_its_named_form() {
    for (rebuild, expected) in REBUILDS {
        assert_eq!(
            text(&change(b"carbon-prod", TargetChange::Changed { parts: Box::new([]), rebuild })).as_deref(),
            Ok(expected.as_bytes()),
            "{rebuild:?}"
        );
    }
}
fn change(target: &[u8], change: TargetChange) -> Record {
    Record::Change(Change { target: copy_of(target), change })
}
fn text(record: &Record) -> Result<Box<[u8]>, form::Refusal> {
    form::print(Form::Text, record, Some(Type::Change), &limits::SHIPPED)
}
fn terminal(targets: u32, changed: u32, unchanged: u32) -> Record {
    Record::Terminal(Terminal { outcome: Outcome::Negative, diff: Some(fixtures::tally(targets, changed, unchanged)) })
}
fn hunks(value: &mut Change) -> &mut Hunks {
    match &mut value.change {
        TargetChange::Changed { parts, .. } => match parts.first_mut().expect("config part") {
            PartChange::Config(config) => match config.tree.first_mut().expect("file change") {
                EntryChange::Changed { hunks, .. } => hunks.as_mut().expect("fixture hunks"),
                EntryChange::Added { .. } | EntryChange::Removed { .. } => unreachable!("changed file fixture"),
            },
        },
        TargetChange::Added { .. } | TargetChange::Removed | TargetChange::Same | TargetChange::Failed { .. } => {
            unreachable!("changed fixture")
        }
    }
}
