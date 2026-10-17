use super::fixtures::{self, ARCHIVE};
use crate::hearth::{
    draw::Draw,
    mirror::Timestamp,
    universe::{Priority, Universe},
};
use skein_lib::Rng;

const IN_RELEASE: &[u8] = b"Origin: Debian\nLabel: Debian\nSuite: testing\nCodename: forky\nArchitectures: amd64\nComponents: main\nSHA256:\n 6e80ba38d7802345c39dc883a60851c8e1c9ddbc9911cffa6861d761f1253ed2 306 main/binary-amd64/Packages\n";
#[test]
fn a_suite_s_inrelease_lists_its_index_by_sha256_and_size() {
    let suite = fixtures::forky(Universe::read(fixtures::MAWK_KEPT).expect("fixture reads"));
    assert_eq!(suite.in_release(), IN_RELEASE);
    assert_eq!(suite.index(b"main"), Some(fixtures::MAWK_KEPT));
}
const SNAPSHOTS: &[(&[u8], Option<&[u8]>)] = &[
    (b"20261017T143929Z", Some(b"1.3.4.20260302-2")),
    (b"20261018T000000Z", Some(b"1.3.4.20260302-2")),
    (b"20261020T000000Z", Some(b"9.0")),
    (b"20261025T000000Z", Some(b"9.0")),
];
#[test]
fn a_timestamp_between_two_snapshots_answers_as_the_earlier() {
    let mirror = fixtures::mirror();
    for &(timestamp, expected) in SNAPSHOTS {
        let package = mirror
            .suite(ARCHIVE, fixtures::at(timestamp), b"forky")
            .and_then(|s| s.universe(b"main"))
            .and_then(|u| u.package(b"mawk"));
        assert_eq!(package.map(|p| &*p.version), expected, "{timestamp:?}");
    }
}
#[test]
fn a_timestamp_before_the_first_snapshot_finds_no_suite() {
    let mirror = fixtures::mirror();
    assert_eq!(mirror.suite(ARCHIVE, fixtures::at(b"20000101T000000Z"), b"forky"), None);
}
const UNAVAILABLE: &[(&[u8], &[u8])] = &[(b"ubuntu", b"forky"), (b"debian", b"missing")];
#[test]
fn an_archive_or_suite_the_mirror_lacks_is_unavailable() {
    let mirror = fixtures::mirror();
    for &(archive, suite) in UNAVAILABLE {
        assert_eq!(mirror.suite(archive, fixtures::at(b"20261025T000000Z"), suite), None);
    }
}
const BUMPS: &[(u32, &[u8])] = &[(0, b"9.0"), (1000, b"9.0+b1")];
#[test]
fn a_bump_moves_every_version_at_1000_none_at_0_and_keeps_the_earlier_snapshot() {
    for &(per_mille, expected) in BUMPS {
        let mut mirror = fixtures::mirror();
        mirror.bump(&mut Rng::new(fixtures::SEED), fixtures::at(b"20261101T000000Z"), per_mille);
        let suite = mirror.suite(ARCHIVE, fixtures::at(b"20261101T000000Z"), b"forky").expect("bumped suite");
        assert_eq!(&*suite.universe(b"main").expect("main").package(b"mawk").expect("mawk").version, expected);
        let earlier = mirror.suite(ARCHIVE, fixtures::at(b"20261020T000000Z"), b"forky").expect("earlier remains");
        assert_eq!(&*earlier.universe(b"main").expect("main").package(b"mawk").expect("mawk").version, b"9.0");
    }
}
const BUMPED_SHA: [u8; 32] = [
    0x14, 0x1c, 0x69, 0x5e, 0x5a, 0xaa, 0x85, 0xe9, 0x37, 0x6e, 0x6d, 0x22, 0x93, 0x95, 0xee, 0x7c, 0x8f, 0x88, 0x6b,
    0x1d, 0x5b, 0xc9, 0x69, 0x4c, 0x70, 0xe6, 0x0d, 0xe8, 0x40, 0xde, 0x4e, 0x71,
];
#[test]
fn a_bump_gives_the_changed_archive_a_new_size_and_digest() {
    let mut mirror = fixtures::mirror();
    mirror.bump(&mut Rng::new(fixtures::SEED), fixtures::at(b"20261101T000000Z"), 1000);
    let package = mirror
        .suite(ARCHIVE, fixtures::at(b"20261101T000000Z"), b"forky")
        .expect("suite")
        .universe(b"main")
        .expect("main")
        .package(b"mawk")
        .expect("mawk");
    assert!(package.size > 144_700, "the changed archive has a new size");
    assert_eq!(package.sha256, BUMPED_SHA);
}
const CORE: &[(&[u8], Priority, bool)] = &[
    (b"base-files", Priority::Required, true),
    (b"base-passwd", Priority::Required, true),
    (b"dpkg", Priority::Required, true),
    (b"libc6", Priority::Required, false),
    (b"mawk", Priority::Required, false),
    (b"gawk", Priority::Optional, false),
    (b"debconf", Priority::Required, false),
    (b"apt", Priority::Required, false),
    (b"extra-one", Priority::Optional, false),
    (b"extra-two", Priority::Optional, false),
];
#[test]
fn a_draw_holds_its_core_and_every_name_it_asks_for() {
    let universe =
        Universe::draw(&mut Rng::new(fixtures::SEED), &Draw { packages: 24, names: &[b"extra-one", b"extra-two"] });
    for &(name, priority, essential) in CORE {
        let package = universe.package(name).expect("the requested name is present");
        assert_eq!((package.priority, package.essential), (priority, essential), "{name:?}");
    }
    assert_eq!(universe.packages().len(), 35);
}
#[test]
fn one_seed_draws_one_universe() {
    let draw = Draw { packages: 24, names: &[] };
    let first = Universe::draw(&mut Rng::new(fixtures::SEED), &draw);
    let second = Universe::draw(&mut Rng::new(fixtures::SEED), &draw);
    assert_eq!(first, second);
}
const PORTS: &[(Option<u16>, bool)] = &[(None, true), (Some(8123), true), (Some(8124), false)];
#[test]
fn only_the_proxy_port_the_mirror_is_served_on_reaches_it() {
    let mut machine = fixtures::whole();
    machine.serve(fixtures::mirror(), Some(8123));
    for &(port, reaches) in PORTS {
        assert_eq!(machine.reach(port).is_some(), reaches, "{port:?}");
    }
}
#[test]
fn taking_the_mirror_back_leaves_no_server() {
    let mut machine = fixtures::whole();
    machine.serve(fixtures::mirror(), Some(8123));
    assert!(machine.unserve().is_some(), "the world takes ownership back");
    assert_eq!(machine.reach(None), None);
    assert_eq!(machine.unserve(), None);
}
const TIMESTAMPS: &[(&[u8], bool)] = &[
    (b"20261017T143929Z", true),
    (b"20000101T000000Z", true),
    (b"20261017X143929Z", false),
    (b"20261017T143929", false),
    (b"20261017T1x3929Z", false),
];
#[test]
fn a_timestamp_requires_the_snapshot_spelling() {
    for &(bytes, expected) in TIMESTAMPS {
        assert_eq!(Timestamp::new(bytes).is_some(), expected, "{bytes:?}");
    }
}
