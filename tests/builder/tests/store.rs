use hearth_builder_world::fixtures::{self, Release, Repository};
use hearth_domain_store as store;
use hearth_protocol_tree::{limits, store as record};
use skein_lib::Duration;
use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
};
#[test]
fn a_render_over_a_locked_store_waits_then_renders_once_it_is_released() {
    let repository = Repository::make("locked", fixtures::THIN);
    // Five asks fit the test: shipped retry is 200 ms.
    let limits =
        fixtures::with_store(store::limits::Limits { retry: Duration::from_millis(20), ..store::limits::SHIPPED });
    let held = fixtures::render_holding(&repository, &limits, Release::After(Duration::from_millis(100)));
    assert!(held.printed_while_held.is_empty(), "nothing renders while the lock is busy");
    assert_eq!(String::from_utf8_lossy(&held.rendered.stdout), fixtures::THIN_TEXT);
}
#[test]
fn the_first_render_records_the_store_as_layout_1_of_its_user() {
    let repository = Repository::make("first-record", fixtures::THIN);
    drop(fixtures::render_with(&repository, &hearth::limits()));
    let store = repository.scratch().path().join("store");
    let record =
        record::read(&fs::read(store.join("store.record")).expect("store record"), &limits::SHIPPED).expect("kind 1");
    assert_eq!(record.layout, 1);
    assert_eq!(record.range, None);
    assert_eq!(record.uid, fs::metadata(&store).expect("store owner").uid());
    let mut names: Vec<_> =
        fs::read_dir(&store).expect("top").map(|entry| entry.expect("top entry").file_name()).collect();
    names.sort();
    assert_eq!(names, ["cache", "resolutions", "staging", "store.record"]);
    for name in ["cache", "resolutions", "staging"] {
        assert_eq!(fs::metadata(store.join(name)).expect("directory").permissions().mode() & 0o7777, 0o700, "{name}");
    }
}
#[test]
fn a_second_render_keeps_the_store_record_it_found() {
    let repository = Repository::make("second-record", fixtures::THIN);
    drop(fixtures::render_with(&repository, &hearth::limits()));
    let path = repository.scratch().path().join("store/store.record");
    let bytes = fs::read(&path).expect("first record");
    drop(fixtures::render_with(&repository, &hearth::limits()));
    assert_eq!(fs::read(path).expect("kept record"), bytes);
}
#[test]
fn an_unreadable_store_record_is_written_again() {
    let repository = Repository::make("unreadable-record", fixtures::THIN);
    drop(fixtures::render_with(&repository, &hearth::limits()));
    let path = repository.scratch().path().join("store/store.record");
    let mut bytes = fs::read(&path).expect("first record");
    *bytes.get_mut(11).expect("version byte") = 2;
    fs::write(&path, bytes).expect("newer version");
    drop(fixtures::render_with(&repository, &hearth::limits()));
    let record = record::read(&fs::read(path).expect("rewritten record"), &limits::SHIPPED).expect("version 1 again");
    assert_eq!(record.layout, 1);
    assert_eq!(record.range, None);
}

const CRASH: &[fixtures::Laid] = &[
    fixtures::Laid::File("cache/0101010101010101010101010101010101010101010101010101010101010101"),
    fixtures::Laid::Record(
        "cache/0101010101010101010101010101010101010101010101010101010101010101.record",
        fixtures::D1,
    ),
    fixtures::Laid::File("cache/0202020202020202020202020202020202020202020202020202020202020202"),
    fixtures::Laid::Record(
        "cache/0303030303030303030303030303030303030303030303030303030303030303.record",
        fixtures::D3,
    ),
    fixtures::Laid::File("cache/0101010101010101010101010101010101010101010101010101010101010101.tmp"),
    fixtures::Laid::File(
        "cache/0101010101010101010101010101010101010101010101010101010101010101.record.skein-0123456789abcdef",
    ),
    fixtures::Laid::File("cache/notes"),
];
#[test]
fn the_scan_settles_what_a_crash_left_in_the_cache() {
    let repository = Repository::make("crash-cache", fixtures::THIN);
    let store = repository.scratch().path().join("store");
    fixtures::lay(&store, CRASH);
    let rendered = fixtures::render_with(&repository, &hearth::limits());
    assert_eq!(rendered.exit, skein_io::kernel::Exit::Code(0));
    assert_eq!(
        fixtures::names(&store.join("cache")),
        [
            "0101010101010101010101010101010101010101010101010101010101010101",
            "0101010101010101010101010101010101010101010101010101010101010101.record",
            "775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123",
            "f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745",
            "notes",
        ]
    );
}
#[test]
fn the_scan_removes_the_store_records_temporary() {
    let repository = Repository::make("crash-store-record", fixtures::THIN);
    let store = repository.scratch().path().join("store");
    fixtures::lay(&store, &[fixtures::Laid::File("store.record.skein-0123456789abcdef")]);
    drop(fixtures::render_with(&repository, &hearth::limits()));
    assert_eq!(fixtures::names(&store), ["cache", "resolutions", "staging", "store.record"]);
}
