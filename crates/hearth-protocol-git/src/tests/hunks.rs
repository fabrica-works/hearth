use super::fixtures;
use crate::{
    hunks::{Error, Event, Kind},
    limits::{self, Limits},
};
const LINES: &[(Kind, &[u8])] = &[(Kind::Hunk, b"@@ -1 +1 @@"), (Kind::Removed, b"a"), (Kind::Added, b"b")];
#[test]
fn the_headers_before_the_first_hunk_are_skipped() {
    let Event::Hunks(hunks) =
        fixtures::hunked(b"diff --git a/a b/b\n--- a/a\n+++ b/b\n@@ -1 +1 @@\n-a\n+b\n", &limits::SHIPPED)
    else {
        unreachable!("valid hunks")
    };
    assert_eq!(hunks.lines.len(), 3, "only hunk lines");
    for (line, expected) in hunks.lines.iter().zip(LINES) {
        assert_eq!((line.kind, line.text.as_ref()), *expected);
    }
}
const KINDS: &[(Kind, &[u8])] = &[
    (Kind::Hunk, b"@@ -1 +1 @@"),
    (Kind::Context, b"context"),
    (Kind::Removed, b"removed"),
    (Kind::Added, b"added"),
    (Kind::NoNewline, b" No newline at end of file"),
];
#[test]
fn each_hunk_line_decodes_to_its_kind_and_text() {
    let Event::Hunks(hunks) =
        fixtures::hunked(b"@@ -1 +1 @@\n context\n-removed\n+added\n\\ No newline at end of file\n", &limits::SHIPPED)
    else {
        unreachable!("valid lines")
    };
    assert_eq!(hunks.lines.len(), 5);
    for (line, expected) in hunks.lines.iter().zip(KINDS) {
        assert_eq!((line.kind, line.text.as_ref()), *expected);
    }
}
#[test]
fn lines_past_the_limit_are_counted_never_kept() {
    let limits = Limits { hunk_lines: 2, ..limits::SHIPPED };
    let Event::Hunks(hunks) = fixtures::hunked(b"@@ -1 +1 @@\n-a\n+b\n", &limits) else {
        unreachable!("valid limited hunks")
    };
    assert_eq!(hunks.lines.len(), 2);
    assert_eq!(hunks.left_out, 1);
}
#[test]
fn a_binary_files_line_answers_binary_with_no_line() {
    let Event::Hunks(hunks) = fixtures::hunked(b"Binary files a/a and b/b differ\n", &limits::SHIPPED) else {
        unreachable!("binary diff")
    };
    assert!(hunks.binary);
    assert_eq!(hunks.lines.as_ref(), []);
    assert_eq!(hunks.left_out, 0);
}
const NO_HUNKS: &[&[u8]] = &[b"", b"diff --git a/a b/b\n--- a/a\n+++ b/b\n"];
#[test]
fn an_output_with_no_hunk_and_no_binary_line_fails() {
    for stdout in NO_HUNKS {
        assert_eq!(fixtures::hunked(stdout, &limits::SHIPPED), Event::Failed(Error::Malformed));
    }
}
#[test]
fn a_line_longer_than_a_chunk_is_read_whole() {
    let limits = Limits { chunk: 2, ..limits::SHIPPED };
    let Event::Hunks(hunks) = fixtures::hunked(b"@@ -1 +1 @@\n-a\n+b\n", &limits) else {
        unreachable!("chunked lines")
    };
    assert_eq!(hunks.lines.len(), 3);
    for (line, expected) in hunks.lines.iter().zip(LINES) {
        assert_eq!((line.kind, line.text.as_ref()), *expected);
    }
}
#[test]
fn a_line_past_its_file_limit_fails() {
    let limits = Limits { hunk_file: 12, ..limits::SHIPPED };
    assert_eq!(fixtures::hunked(b"@@ -1 +1 @@\n+123456789012\n", &limits), Event::Failed(Error::TooLarge));
}
#[test]
fn retained_text_past_twice_the_file_limit_fails() {
    let limits = Limits { hunk_file: 12, ..limits::SHIPPED };
    assert_eq!(fixtures::hunked(b"@@ -1 +1 @@\n+12345678901\n+12345678901\n", &limits), Event::Failed(Error::TooLarge));
}
