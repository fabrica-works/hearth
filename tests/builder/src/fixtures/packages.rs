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
pub const CMDLINE: &[File] = &[
    File { path: "tasks/params/task.json", body: br#"{"cmdline": {"console": {"param": "console=ttyS0,115200n8", "order": 900}, "audit": {"param": "audit=0"}, "quiet": {"param": "quiet", "order": 100}}}"#, mode: 0o644 },
    File { path: "hosts/x1/x1.json", body: br#"{"motd": {}, "params": {}}"#, mode: 0o644 },
];
pub const CMDLINE_ORDER: File = File {
    path: "tasks/params/task.json",
    body: br#"{"cmdline": {"loud": {"param": "debug", "order": 1000}}}"#,
    mode: 0o644,
};
#[must_use]
pub fn command_line_repository(name: &str, order_fault: bool) -> Repository {
    let mut files = thin_with(CMDLINE);
    if order_fault {
        for file in &mut files {
            if file.path == CMDLINE_ORDER.path {
                *file = CMDLINE_ORDER;
            }
        }
    }
    Repository::make(name, &files)
}

#[must_use]
pub fn packages_diff(name: &str) -> super::Rendered {
    let repository = packages_repository(name, true);
    let from = repository.head();
    let to = repository.next(&[File { path: "tasks/base/task.json", body: br#"{"packages": {"curl": {}, "tzdata": {}, "vim": {}, "firmware-linux": {"component": "non-free-firmware"}}}"#, mode: 0o644 }], &[]);
    super::diff(&repository, &super::Asked { form: hearth::Form::JsonLines, ..super::Asked::new(&from, &to) })
}
#[must_use]
pub fn boot_diff(name: &str) -> super::Rendered {
    let repository = command_line_repository(name, false);
    let from = repository.head();
    let to = repository.next(&[File { path: "tasks/params/task.json", body: br#"{"cmdline": {"console": {"param": "console=ttyS0,115200n8", "order": 900}, "audit": {"param": "audit=0"}}}"#, mode: 0o644 }], &[]);
    super::diff(&repository, &super::Asked { form: hearth::Form::JsonLines, ..super::Asked::new(&from, &to) })
}
