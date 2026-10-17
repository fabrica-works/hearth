use hearth_builder_world::fixtures::{
    self,
    file_world::{self as files, ABC_SHA256, ABC_SIZE, OTHER_SHA256},
};
use skein_io::file::{Event, Expect};

struct Store {
    name: &'static str,
    laid: bool,
    expected: Expect,
    terminal: Event,
}
const STORES: &[Store] = &[
    Store { name: "any missing", laid: false, expected: Expect::Any, terminal: files::ABC_STORED },
    Store { name: "any existing", laid: true, expected: Expect::Any, terminal: files::ABC_STORED },
    Store { name: "absent missing", laid: false, expected: Expect::Absent, terminal: files::ABC_STORED },
    Store {
        name: "absent existing",
        laid: true,
        expected: Expect::Absent,
        terminal: Event::Conflict { owner: fixtures::WORLD_OWNER, now: Some(ABC_SHA256) },
    },
    Store { name: "digest agrees", laid: true, expected: Expect::Digest(ABC_SHA256), terminal: files::ABC_STORED },
    Store {
        name: "digest differs",
        laid: true,
        expected: Expect::Digest(OTHER_SHA256),
        terminal: Event::Conflict { owner: fixtures::WORLD_OWNER, now: Some(ABC_SHA256) },
    },
    Store {
        name: "digest missing",
        laid: false,
        expected: Expect::Digest(ABC_SHA256),
        terminal: Event::Conflict { owner: fixtures::WORLD_OWNER, now: None },
    },
];
#[test]
fn a_whole_file_store_rechecks_its_expectation() {
    for row in STORES {
        let mut files = files::files(row.laid);
        assert_eq!(files::store(&mut files, row.expected), row.terminal, "{}", row.name);
    }
}

#[test]
fn a_store_closes_and_removes_its_temporary_after_a_conflict() {
    let mut files = files::files(true);
    drop(files::store(&mut files, Expect::Absent));
    assert_eq!(
        files.names(files.roots.cache).iter().map(|entry| entry.name.as_ref()).collect::<Vec<_>>(),
        [files::FILE_NAME]
    );
}

#[test]
fn a_whole_file_load_reads_the_laid_bytes() {
    assert_eq!(
        files::load(&mut files::files(true), ABC_SIZE),
        Event::Loaded { owner: fixtures::WORLD_OWNER, bytes: b"abc".as_slice().into() }
    );
}

#[test]
fn a_load_past_its_bound_reports_the_size_without_reading() {
    let mut files = files::files(true);
    assert_eq!(
        files::load(&mut files, ABC_SIZE - 1),
        Event::TooLarge { owner: fixtures::WORLD_OWNER, size: u64::from(ABC_SIZE) }
    );
    assert_eq!(
        files.shared.borrow().ledger.iter().map(|entry| entry.bytes).sum::<u64>(),
        0,
        "stat and close without reading content"
    );
}
