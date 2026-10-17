use super::{File, Repository, thin_with};
pub const PACKAGES: &[File] = &[
    File {
        path: "tasks/base/task.json",
        body: br#"{"packages": {"curl": {}, "tzdata": {}, "firmware-linux": {"component": "non-free-firmware"}}}"#,
        mode: 0o644,
    },
    File {
        path: "hosts/x1/x1.json",
        body: br#"{"motd": {}, "base": {}, "debian": {"components": {"non-free-firmware": true}}}"#,
        mode: 0o644,
    },
];
pub const COMPONENT_OFF: File = File { path: "hosts/x1/x1.json", body: br#"{"motd": {}, "base": {}}"#, mode: 0o644 };
#[must_use]
pub fn packages_repository(name: &str, enabled: bool) -> Repository {
    let mut files = thin_with(PACKAGES);
    if !enabled {
        for file in &mut files {
            if file.path == COMPONENT_OFF.path {
                *file = COMPONENT_OFF;
            }
        }
    }
    Repository::make(name, &files)
}
pub const PRESEEDED: &[File] = &[
    File { path: "tasks/base/task.json", body: br#"{"packages": {"curl": {}, "tzdata": {}}, "preseeds": {"tzdata/Areas": {"type": "select", "value": "Europe"}}, "initramfs": {"btrfs": {}, "nvme": {}}}"#, mode: 0o644 },
    COMPONENT_OFF,
];
