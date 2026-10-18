use super::{Asked, File, Form, Rendered, Repository};
#[derive(Clone, Copy, Debug)]
pub enum Part {
    Software,
    Secrets,
    Live,
    Seeds,
}
pub const PARTS: &[Part] = &[Part::Software, Part::Secrets, Part::Live, Part::Seeds];
pub const SOFTWARE_MOVED: File = File { path: "tasks/motd/task.json", mode: 0o644, body: br#"{"software": {"forgejo": {"url": "https://codeberg.org/forgejo/forgejo/releases/download/v15.0.0/forgejo-15.0.0-linux-amd64", "sha256": "7b36e8a38247d55d927349bd9083ffa10c819d2fedab7ce32be0943a760747ed", "unpack": "file", "path": "/opt/forgejo/bin/forgejo"}, "ripgrep": {"url": "https://github.com/BurntSushi/ripgrep/releases/download/15.1.1/ripgrep-15.1.1-x86_64-unknown-linux-musl.tar.gz", "sha256": "d2571cab39973c001a1734a8c432acb9111d135a97958989ba504dc8241d8bde", "unpack": "tar", "strip": 1, "path": "/opt/ripgrep"}}, "scripts": {"software.sh": {"order": 500}}}"# };
pub const SECRETS_MOVED: File = File { path: "tasks/motd/task.json", body: br#"{"secrets": {"mail/dkim/example.org.key": {"path": "/etc/dkimkeys/example.org.key", "mode": "0640", "group": "opendkim"}, "mail/tls/mx1.key": {"path": "/etc/ssl/private/mx1.key", "mode": "0640"}}, "identity": {"ssh-ed25519": {"kind": "ssh-host-key", "algorithm": "ed25519", "path": "/etc/ssh/ssh_host_ed25519_key", "public_path": "/etc/ssh/ssh_host_ed25519_key.pub"}}, "users": {"free": {"uid": 1000, "password": "users/free2.passwd"}}, "groups": {"opendkim": {"from": "package", "gid": 120}}}"#, mode: 0o644 };
pub const SECRETS_FIXED_MOVED: File = File { path: "tasks/motd/task.json", body: br#"{"secrets": {"mail/dkim/example.org.key": {"path": "/etc/dkimkeys/example.org.key", "mode": "0640", "group": "opendkim"}, "mail/tls/mx1.key": {"path": "/etc/ssl/private/mx1.key"}}, "identity": {"ssh-ed25519": {"kind": "ssh-host-key", "algorithm": "ed25519", "path": "/etc/ssh/ssh_host_ed25519_key", "public_path": "/etc/ssh/ssh_host_ed25519_key.pub"}}, "users": {"free": {"uid": 1000, "password": "users/free.passwd"}}, "groups": {"opendkim": {"from": "package", "gid": 121}}}"#, mode: 0o644 };
pub const LIVE_MOVED: File = File { path: "tasks/motd/task.json", mode: 0o644,
    body: br#"{"live":{"postfix":{"unit":"postfix.service","action":"reload","paths":["/etc/postfix/","/etc/mailname"]},"ripgrep":{"action":"none","paths":["/opt/ripgrep/"]},"nginx":{"unit":"nginx.service","action":"reload","paths":["/etc/nginx/"]}}}"# };
#[must_use]
pub fn changed(part: Part, form: Form) -> Rendered {
    let (name, before, after) = match part {
        Part::Software => ("software-change", super::software::SOFTWARE, SOFTWARE_MOVED),
        Part::Secrets => ("secrets-change", core::slice::from_ref(&super::SECRETS), SECRETS_MOVED),
        Part::Live => ("live-change", core::slice::from_ref(&super::LIVE), LIVE_MOVED),
        Part::Seeds => ("seeds-change", core::slice::from_ref(&super::ids::SECRETS_FIXED), SECRETS_FIXED_MOVED),
    };
    let repository = Repository::make(name, &super::thin_with(before));
    let from = repository.head();
    let to = repository.next(&[after], &[]);
    super::diff(&repository, &Asked { form, ..Asked::new(&from, &to) })
}
