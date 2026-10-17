use super::fixtures;
use crate::Pin;
use std::fs;

const NAMES: &[&str] = &["test-min", "test-boot"];
const TARGETS: &[(&str, &str, &[&str])] = &[
    ("test-min", "apt", &[]),
    (
        "test-boot",
        "apt",
        &[
            "efibootmgr",
            "git",
            "incus-agent",
            "linux-image-amd64",
            "mmdebstrap",
            "openssh-server",
            "systemd",
            "systemd-container",
            "tiny-initramfs",
            "uidmap",
        ],
    ),
];
#[test]
fn the_committed_pin_names_test_min_and_test_boot() {
    let pin = Pin::committed();
    let names: Vec<&str> = pin.targets.iter().map(|target| &*target.name).collect();
    assert_eq!(names, NAMES);
    for &(name, variant, request) in TARGETS {
        let target = pin.target(name).unwrap();
        let names: Vec<&str> = target.request.iter().map(AsRef::as_ref).collect();
        assert_eq!((&*target.variant, names.as_slice()), (variant, request), "{name}");
    }
}
const ARCHIVE: &str = "20261017T143929Z";
const PINNED_ARCHIVE: &str = "\"archive\": \"20261017T143929Z\"";
#[test]
fn the_test_repository_names_the_pin_s_snapshot_timestamp() {
    let pin = Pin::committed();
    let fleet = fs::read_to_string(fixtures::repository("test").join("fleet.json")).unwrap();
    assert_eq!(&*pin.archive, ARCHIVE);
    assert!(fleet.contains(PINNED_ARCHIVE), "debian.snapshot.archive is the committed pin's timestamp");
}
const BOOT_PACKAGES: &[&str] = &[
    "\"efibootmgr\": {}",
    "\"git\": {}",
    "\"incus-agent\": {}",
    "\"linux-image-amd64\": {}",
    "\"mmdebstrap\": {}",
    "\"openssh-server\": {}",
    "\"systemd\": {}",
    "\"systemd-container\": {}",
    "\"tiny-initramfs\": {}",
    "\"uidmap\": {}",
];
#[test]
fn the_boot_task_declares_test_boot_s_request() {
    let boot = fs::read_to_string(fixtures::repository("test").join("tasks/boot/task.json")).unwrap();
    for package in BOOT_PACKAGES {
        assert!(boot.contains(package), "the boot task declares {package}");
    }
}
const FILES: &[&str] = &[
    "envs/boot.json",
    "envs/min.json",
    "fleet.json",
    "hosts/test/test-boot.json",
    "hosts/test/test-min.json",
    "hosts/test/test.json",
    "tasks/boot/task.json",
    "tasks/hearth/task.json",
];
#[test]
fn the_test_repository_holds_the_eight_files_above_and_nothing_else() {
    assert_eq!(fixtures::files(&fixtures::repository("test")), FILES);
}
