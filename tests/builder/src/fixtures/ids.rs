use super::{File, Form, Rendered, Repository};
pub const SECRETS_FIXED: File = File { path: "tasks/motd/task.json", body: br#"{"secrets": {"mail/dkim/example.org.key": {"path": "/etc/dkimkeys/example.org.key", "mode": "0640", "group": "opendkim"}, "mail/tls/mx1.key": {"path": "/etc/ssl/private/mx1.key"}}, "identity": {"ssh-ed25519": {"kind": "ssh-host-key", "algorithm": "ed25519", "path": "/etc/ssh/ssh_host_ed25519_key", "public_path": "/etc/ssh/ssh_host_ed25519_key.pub"}}, "users": {"free": {"uid": 1000, "password": "users/free.passwd"}}, "groups": {"opendkim": {"from": "package", "gid": 120}}}"#, mode: 0o644 };
pub const SECRETS_NO_ID: File = File {
    path: "tasks/motd/task.json",
    body: br#"{"secrets": {"mail/tls/mx1.key": {"path": "/etc/ssl/private/mx1.key", "owner": "postfix"}}}"#,
    mode: 0o644,
};
#[must_use]
pub fn no_id() -> Rendered {
    let repository = Repository::make("secrets-no-id", &super::thin_with(core::slice::from_ref(&SECRETS_NO_ID)));
    super::render(&repository, &[], Form::Text)
}
