use super::{File, Form, Rendered, Repository};
pub const WAIT: File = task(br#"{"cmdline": {"wait": {"param": "rootwait"}}}"#);
pub const PASSWORD: File = task(br#"{"preseeds": {"d/p": {"type": "password", "value": "x"}}}"#);
pub const CONFIGURATION_EXIT: skein_io::kernel::Exit = skein_io::kernel::Exit::Code(2); // client/README.md, 3.3.
pub const PARAMETER_BYTES: usize = 1000; // Two parameters make 139 + 2 * (1 + 1000) = 2141 bytes.
#[must_use]
pub fn long_params() -> String {
    format!(
        "{{\"cmdline\": {{\"a\": {{\"param\": \"{}\"}}, \"b\": {{\"param\": \"{}\"}}}}}}",
        "a".repeat(PARAMETER_BYTES),
        "b".repeat(PARAMETER_BYTES)
    )
}
#[must_use]
pub fn rendered(file: &File) -> Rendered {
    rendered_files(core::slice::from_ref(file))
}
#[must_use]
pub fn long_line() -> Rendered {
    let repository = Repository::make("reserved-length", super::THIN);
    std::fs::write(repository.work_tree().join("tasks/motd/task.json"), long_params()).expect("long parameters");
    drop(repository.git(&["add", "-A"]));
    drop(repository.git(&["commit", "-q", "-m", "parameters"]));
    rendered_repository(&repository)
}
pub const KEY_IN_TREE: &[File] = &[
    task(br#"{"files": {"/etc/ssl/private/mx1.key": {}}}"#),
    File { path: "tasks/motd/tree/etc/ssl/private/mx1.key", body: b"key\n", mode: 0o644 },
];
pub const FSTAB: File = File { path: "tasks/motd/tree/etc/fstab", body: b"fstab\n", mode: 0o644 };
pub const DATA_PATH: File = task(br#"{"data": {"mail": {"path": "/srv/data/spool"}}}"#);
#[must_use]
pub fn rendered_files(files: &[File]) -> Rendered {
    let repository = Repository::make("reserved-path", &super::thin_with(files));
    rendered_repository(&repository)
}

#[must_use]
fn rendered_repository(repository: &Repository) -> Rendered {
    super::render(repository, &[], Form::Text)
}
pub const SECRET_HEARTH: File =
    task(br#"{"secrets": {"mail/tls/mx1.key": {"path": "/etc/ssl/private/mx1.key", "owner": "hearth"}}}"#);
pub const LIVE_NEVER: File = task(
    br#"{"live": {"net": {"unit": "systemd-networkd.service", "action": "restart", "paths": ["/etc/hostname"]}}}"#,
);
pub const LIVE_NONE: File =
    task(br#"{"live": {"rg": {"unit": "x.service", "action": "none", "paths": ["/opt/rg/"]}}}"#);

#[must_use]
const fn task(body: &'static [u8]) -> File {
    File { path: "tasks/motd/task.json", body, mode: 0o644 }
}
