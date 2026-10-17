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
            "775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123.record",
            "f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745",
            "f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745.record",
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

const STATIC: &[(&str, u64, &[u8])] = &[
    (
        "775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123",
        15,
        b"d23c59a84714a16a9597f6d92ce69390d6d13f9b",
    ),
    (
        "f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745",
        29,
        b"71318968f52ccfeabf6edd13442c868eaa35e49d",
    ),
];
#[test]
fn each_static_file_enters_the_cache_with_its_record_naming_its_blob() {
    use hearth_protocol_tree::cache_file::{self, Origin};
    let repository = Repository::make("cache-records", fixtures::THIN);
    drop(fixtures::render_with(&repository, &hearth::limits()));
    let cache = repository.scratch().path().join("store/cache");
    for &(name, size, oid) in STATIC {
        let bytes = fs::read(cache.join(format!("{name}.record"))).expect("cache record");
        let record = cache_file::read(&bytes, &limits::SHIPPED).expect("kind 5");
        assert_eq!(record.sha256.0.map(|byte| format!("{byte:02x}")).concat(), name, "{name}");
        assert_eq!(record.size, size, "{name}");
        assert_eq!(record.origin, Origin::Blob { oid: oid.into() }, "{name}");
        assert_ne!(record.at, skein_lib::Wall::EPOCH, "{name}");
        assert_ne!(record.took, Duration::ZERO, "{name}");
    }
}
#[test]
fn a_file_already_in_the_cache_keeps_its_first_record() {
    let repository = Repository::make("first-cache-record", fixtures::THIN);
    drop(fixtures::render_with(&repository, &hearth::limits()));
    let cache = repository.scratch().path().join("store/cache");
    let records: Vec<_> = STATIC
        .iter()
        .map(|(name, _, _)| fs::read(cache.join(format!("{name}.record"))).expect("first record"))
        .collect();
    drop(fixtures::render_with(&repository, &hearth::limits()));
    for ((name, _, _), bytes) in STATIC.iter().zip(records) {
        assert_eq!(fs::read(cache.join(format!("{name}.record"))).expect("kept record"), bytes, "{name}");
    }
}
const FULL_CACHE: &[fixtures::Laid] = &[
    fixtures::Laid::File("cache/0101010101010101010101010101010101010101010101010101010101010101"),
    fixtures::Laid::Record(
        "cache/0101010101010101010101010101010101010101010101010101010101010101.record",
        fixtures::D1,
    ),
    fixtures::Laid::File("cache/0202020202020202020202020202020202020202020202020202020202020202"),
    fixtures::Laid::Record(
        "cache/0202020202020202020202020202020202020202020202020202020202020202.record",
        hearth::service::domain::render::vocabulary::Digest([2; 32]),
    ),
    fixtures::Laid::File("cache/0303030303030303030303030303030303030303030303030303030303030303"),
    fixtures::Laid::Record(
        "cache/0303030303030303030303030303030303030303030303030303030303030303.record",
        fixtures::D3,
    ),
];
#[test]
fn a_store_past_its_cache_limit_refuses_every_new_file() {
    let repository = Repository::make("full-cache", fixtures::THIN);
    let store = repository.scratch().path().join("store");
    fixtures::lay(&store, FULL_CACHE);
    let rendered = fixtures::render_with(
        &repository,
        &fixtures::with_store(store::limits::Limits { cache_files: 2, ..store::limits::SHIPPED }),
    );
    assert_eq!(rendered.exit, skein_io::kernel::Exit::Code(75));
    assert_eq!(rendered.stdout, b"");
    for &(name, _, _) in STATIC {
        assert!(!store.join("cache").join(name).exists(), "{name}");
    }
    assert_eq!(fixtures::names(&store.join("cache")).len(), 6);
    assert_eq!(
        fs::read(store.join("cache/0101010101010101010101010101010101010101010101010101010101010101"))
            .expect("laid file"),
        b"fixture"
    );
}
#[test]
fn a_cache_that_fills_while_rendering_removes_the_file_past_it() {
    let repository = Repository::make("cache-fills", fixtures::THIN);
    let rendered = fixtures::render_with(
        &repository,
        &fixtures::with_store(store::limits::Limits { cache_files: 1, ..store::limits::SHIPPED }),
    );
    assert_eq!(rendered.exit, skein_io::kernel::Exit::Code(75));
    assert_eq!(
        fixtures::names(&repository.scratch().path().join("store/cache")),
        [
            "775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123",
            "775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123.record",
        ]
    );
}
