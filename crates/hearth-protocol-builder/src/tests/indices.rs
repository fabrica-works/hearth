use super::fixtures::{self, INDEX_NAMES, INDEX_PACKAGES, INRELEASE_NAME, PACKAGES_NAME};
use crate::builder::{
    store::vocabulary::{Origin, Request as StoreRequest},
    vocabulary::{Event as BuilderEvent, Request as RootRequest},
};
use crate::render::{
    resolution::Stanza,
    vocabulary::{Digest, Event},
};
use skein_io::file;
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

#[test]
fn a_kept_index_is_synced_renamed_across_then_cache_synced_then_recorded() {
    let (mut layer, _) = fixtures::Layer::index_arriving();
    layer.client(RootRequest::Store(StoreRequest::Keep {
        arrival: crate::resolve::OWNER,
        record: fixtures::cache_record(Origin::Index { resolution: fixtures::KEY }),
    }));
    assert_eq!(
        layer.asked(),
        Some(crate::Asked::Beneath(crate::Beneath::Rename {
            owner: fixtures::STORE_OWNER,
            from_root: fixtures::INDEX_DIRECTORY,
            from: fixtures::RESOLVE_INRELEASE_NAME.into(),
            to_root: fixtures::roots().cache.fd,
            to: b"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".as_slice().into()
        }))
    );
    assert_eq!(layer.asked(), None, "the next index waits");
    layer.file_up(file::Event::Renamed { owner: fixtures::STORE_OWNER });
    assert_eq!(
        layer.asked(),
        Some(crate::Asked::File(file::Request::SyncDirectory {
            owner: fixtures::STORE_OWNER,
            root: fixtures::roots().cache.fd
        }))
    );
    layer.file_up(file::Event::Synced { owner: fixtures::STORE_OWNER });
    let Some(crate::Asked::File(file::Request::Store { bytes, path, root, .. })) = layer.asked() else {
        panic!("kind 5 write")
    };
    assert_eq!(path.as_ref(), b"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad.record");
    assert_eq!(root, fixtures::roots().cache.token);
    assert_eq!(
        crate::tree::cache_file::read(&bytes, &crate::tree::limits::SHIPPED).expect("kind 5").origin,
        crate::tree::cache_file::Origin::Index { resolution: fixtures::KEY }
    );
    layer.file_up(file::Event::Stored { owner: fixtures::STORE_OWNER, digest: skein_io::digest::digest(b"record") });
    assert_eq!(layer.told(), Some(BuilderEvent::Store(crate::builder::store::vocabulary::Event::Done)));
}

#[test]
fn a_discarded_index_is_left_for_the_walk() {
    use crate::builder::{
        store::vocabulary::{Event, Request},
        vocabulary::{Event as RootEvent, Request as RootRequest},
    };
    let (mut layer, _) = fixtures::Layer::index_arriving();
    layer.client(RootRequest::Store(Request::Discard { arrival: crate::resolve::OWNER }));
    assert_eq!(layer.asked(), None);
    assert_eq!(layer.told(), Some(RootEvent::Store(Event::Done)));
    let Some(crate::Asked::File(file::Request::OpenReadNoFollow { name, .. })) = layer.asked() else {
        panic!("next index, without removing the discarded file")
    };
    assert!(name.ends_with(fixtures::RESOLVE_PACKAGES_NAME));
}

#[test]
fn each_index_arrives_after_its_file_is_synced_and_closed() {
    use crate::builder::vocabulary::Event;
    let (_, run) = fixtures::Layer::index_arriving();
    assert_eq!(run.trace.as_slice(), ["lists", "InRelease", "sync", "close"]);
    assert_eq!(
        run.events.as_slice(),
        [Event::Index {
            request: fixtures::REQUEST,
            arrival: crate::resolve::OWNER,
            sha256: fixtures::ABC,
            size: 3,
            took: skein_lib::Duration::ZERO
        }]
    );
}
