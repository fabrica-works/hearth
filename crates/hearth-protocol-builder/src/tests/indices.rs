use super::fixtures::{self, INDEX_NAMES, INDEX_PACKAGES, INRELEASE_NAME, PACKAGES_NAME};
use crate::render::{
    resolution::Stanza,
    vocabulary::{Digest, Event},
};
use skein_lib::List;
#[test]
fn the_lists_are_listed_then_each_index_read_in_order() {
    let run = fixtures::read_test_indices();
    assert_eq!(run.steps.as_ref(), fixtures::INDEX_STEPS);
    assert_eq!(run.closed, 3);
}
#[test]
fn a_stanza_split_across_two_reads_reaches_render_whole() {
    let run = fixtures::read_indices(INDEX_NAMES, b"", INDEX_PACKAGES);
    assert_eq!(
        run.events.as_slice(),
        [Event::Stanza {
            request: fixtures::REQUEST,
            line: 0,
            stanza: Stanza {
                package: b"apt".as_slice().into(),
                version: b"1".as_slice().into(),
                architecture: b"all".as_slice().into(),
                filename: b"a".as_slice().into(),
                size: 1,
                sha256: Digest([0; 32]),
            }
        }]
    );
}
#[test]
fn each_index_is_hashed_in_the_pass_that_reads_it() {
    let run = fixtures::read_test_indices();
    let lines = run.result.expect("indices");
    let mut indices = List::with_capacity(2);
    for line in &lines {
        for index in &line.indices {
            indices.push((index.path.as_ref(), index.size, index.sha256)).expect("indices");
        }
    }
    assert_eq!(
        indices.as_slice(),
        [
            (b"InRelease".as_slice(), 3, fixtures::ABC),
            (b"main/binary-amd64/Packages".as_slice(), fixtures::INDEX_PACKAGES_SIZE, fixtures::INDEX_PACKAGES_SHA256),
        ]
    );
    assert_eq!(run.read, 131);
}
#[test]
fn inrelease_is_hashed_and_never_read_for_stanzas() {
    let run = fixtures::read_indices(INDEX_NAMES, INDEX_PACKAGES, b"");
    assert_eq!(run.events.as_slice(), []);
    let lines = run.result.expect("indices");
    let mut digests = List::with_capacity(2);
    for line in &lines {
        for index in &line.indices {
            digests.push(index.sha256).expect("indices");
        }
    }
    assert_eq!(digests.as_slice(), [fixtures::INDEX_PACKAGES_SHA256, fixtures::EMPTY_SHA256]);
}
const INVALID_INDICES: &[&[&[u8]]] = &[
    &[INRELEASE_NAME],
    &[INRELEASE_NAME, b"one_dists_forky_main_binary-amd64_Packages.xz"],
    &[INRELEASE_NAME, PACKAGES_NAME, b"one_dists_forky_main_binary-amd64_Packages.gz"],
];
#[test]
fn a_missing_or_compressed_index_fails_the_run() {
    for (row, names) in INVALID_INDICES.iter().enumerate() {
        assert_eq!(fixtures::read_indices(names, b"abc", INDEX_PACKAGES).result, Err(()), "row {row}");
    }
}
const UNNAMED: &[&[u8]] =
    &[b"elsewhere_Packages", b"elsewhere_InRelease", b"elsewhere_Packages.gz", b"one_dists_forky_InRelease.gz"];
#[test]
fn a_packages_list_no_line_names_fails_the_run() {
    for (row, name) in UNNAMED.iter().enumerate() {
        assert_eq!(
            fixtures::read_indices(&[INRELEASE_NAME, PACKAGES_NAME, name], b"abc", INDEX_PACKAGES).result,
            Err(()),
            "row {row}"
        );
    }
}
#[test]
fn names_that_are_no_index_are_skipped() {
    let run = fixtures::read_indices(&[b"lock", b"partial", INRELEASE_NAME, PACKAGES_NAME], b"abc", INDEX_PACKAGES);
    assert_eq!(run.steps.as_ref(), fixtures::INDEX_STEPS);
}
#[test]
fn a_successful_run_reads_its_indices_before_cleaning_the_scratch_and_resolving() {
    let mut layer = fixtures::Layer::resolution();
    layer.start_resolve();
    layer.resolve_end();
    layer.resolve_exit(skein_io::kernel::Exit::Code(0));
    let run = layer.resolve_with_indices();
    assert_eq!(run.trace.as_slice(), ["lists", "InRelease", "Packages", "scratch", "resolved"]);
    assert_eq!(run.events.len(), 2);
}
