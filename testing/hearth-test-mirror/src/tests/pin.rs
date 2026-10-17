use super::fixtures;
use crate::{Pin, PinError, Wrong};

const SNAPSHOT: (&str, &str, &str) = ("20261016T091523Z", "forky", "amd64");
const TARGETS: &[(&str, &str, &[&str])] = &[("test-min", "apt", &[]), ("test-boot", "apt", &["efibootmgr", "git"])];
#[test]
fn a_pin_reads_its_snapshot_suite_and_targets() {
    let pin = Pin::read(fixtures::PIN).expect("the fixture is in form");
    assert_eq!((&*pin.archive, &*pin.suite, &*pin.architecture), SNAPSHOT);
    for &(name, variant, request) in TARGETS {
        let target = pin.target(name).expect("the target is pinned");
        let names: Vec<&str> = target.request.iter().map(AsRef::as_ref).collect();
        assert_eq!((&*target.variant, names.as_slice()), (variant, request), "{name}");
    }
}
const REFUSED: &[(&[u8], u32, Wrong)] = &[
    (b"unknown x\n", 1, Wrong::Unknown),
    (b"archive 20261016T09152Z\n", 1, Wrong::Form("archive")),
    (b"archive 20261016X091523Z\n", 1, Wrong::Form("archive")),
    (b"archive 20261016T09152XZ\n", 1, Wrong::Form("archive")),
    (b"digest ABCDEF\n", 1, Wrong::Form("digest")),
    (b"digest AD093de033f89b590459503f94d25ea4a36fd044f7ce6d674f43e0d8412161f9\n", 1, Wrong::Form("digest")),
    (b"components contrib main\n", 1, Wrong::Form("components")),
    (b"target test apt git apt\n", 1, Wrong::Form("target")),
    (b"target test apt git git\n", 1, Wrong::Form("target")),
    (b"target test\n", 1, Wrong::Form("target")),
    (b"suite forky\nsuite forky\n", 2, Wrong::Twice),
    (b"suite\n", 1, Wrong::Form("suite")),
    (b"", 0, Wrong::Missing("archive")),
    (b"archive 20261016T091523Z\n", 0, Wrong::Missing("suite")),
    (b"\xff\n", 1, Wrong::Form("text")),
];
#[test]
fn a_pin_out_of_form_is_refused_naming_its_line() {
    for &(text, line, wrong) in REFUSED {
        assert_eq!(Pin::read(text), Err(PinError { line, wrong }), "{text:?}");
    }
}
