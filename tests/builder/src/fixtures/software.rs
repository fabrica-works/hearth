use super::{File, Form, Rendered, Repository};
pub const SOFTWARE: &[File] = &[
    File { path: "tasks/motd/task.json", mode: 0o644, body: br#"{"software": {"forgejo": {"url": "https://codeberg.org/forgejo/forgejo/releases/download/v15.0.0/forgejo-15.0.0-linux-amd64", "sha256": "7b36e8a38247d55d927349bd9083ffa10c819d2fedab7ce32be0943a760747ed", "unpack": "file", "path": "/opt/forgejo/bin/forgejo"}, "ripgrep": {"url": "https://github.com/BurntSushi/ripgrep/releases/download/15.1.0/ripgrep-15.1.0-x86_64-unknown-linux-musl.tar.gz", "sha256": "33165223a0a3c347557831c78e3780cafe5f22ef61a0dbf2a53ef9a3756eb1c9", "unpack": "tar", "strip": 1, "path": "/opt/ripgrep"}}, "scripts": {"software.sh": {"order": 500}}}"# },
    SOFTWARE_SCRIPT,
];
pub const SOFTWARE_OUT: File = File { path: "tasks/motd/task.json", mode: 0o644, body: br#"{"software": {"tool": {"url": "https://codeberg.org/forgejo/forgejo/releases/download/v15.0.0/forgejo-15.0.0-linux-amd64", "sha256": "7b36e8a38247d55d927349bd9083ffa10c819d2fedab7ce32be0943a760747ed", "unpack": "file", "path": "/srv/tool"}}}"# };
#[must_use]
pub fn outside() -> Rendered {
    let repository = Repository::make("software-out", &super::thin_with(&[SOFTWARE_OUT]));
    super::render(&repository, &[], Form::Text)
}

const SOFTWARE_SCRIPT: File = File { path: "tasks/motd/software.sh", mode: 0o644, body: b"rg --version\n" };
pub const BOTH_SCRIPTS: &[File] = &[
    File {
        path: "tasks/motd/task.json",
        mode: 0o644,
        body: br#"{"scripts":{"customize.sh":{"order":300},"software.sh":{"order":500}}}"#,
    },
    SOFTWARE_SCRIPT,
    super::scripts::CUSTOMIZE_SCRIPT,
];
