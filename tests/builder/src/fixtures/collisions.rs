use super::{File, Rendered, Repository};
use hearth::Form;
pub const TREE_TWICE: &[File] = &[
    File { path: "tasks/banner/task.json", mode: 0o644, body: br#"{"files": {"/etc/motd": {"copy": "banner.txt"}}}"# },
    File { path: "banner.txt", mode: 0o644, body: b"Banner\n" },
    File { path: "hosts/x1/x1.json", mode: 0o644, body: br#"{"motd": {}, "banner": {}}"# },
];
pub const HIDDEN: &[File] = &[
    File { path: "tasks/dump/task.json", mode: 0o644, body: b"{}" },
    File { path: "tasks/app/task.json", mode: 0o644, body: br#"{"data": {"app": {"path": "/var/lib/app"}}}"# },
    File { path: "tasks/dump/tree/var/lib/app/seed", mode: 0o644, body: b"seed\n" },
    File { path: "hosts/x1/x1.json", mode: 0o644, body: br#"{"motd": {}, "app": {}, "dump": {}}"# },
];
#[must_use]
pub fn render(name: &str, files: &[File]) -> Rendered {
    let repository = Repository::make(name, &super::thin_with(files));
    super::render(&repository, &[], Form::Text)
}
const USER_A: File = File { path: "tasks/a/task.json", mode: 0o644, body: br#"{"users": {"free": {"uid": 1000}}}"# };
const WITH_USERS: File = File { path: "hosts/x1/x1.json", mode: 0o644, body: br#"{"motd": {}, "a": {}, "b": {}}"# };
pub const CLASH: &[File] = &[
    USER_A,
    WITH_USERS,
    File { path: "tasks/b/task.json", mode: 0o644, body: br#"{"users": {"free": {"uid": 1001}}}"# },
];
pub const SHARED: &[File] = &[
    USER_A,
    WITH_USERS,
    File { path: "tasks/b/task.json", mode: 0o644, body: br#"{"users": {"ci": {"uid": 1000}}}"# },
];
