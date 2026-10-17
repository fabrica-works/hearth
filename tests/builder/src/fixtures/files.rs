use super::{File, Rendered, Repository};
pub const LINKS: &[File] = &[
    File {
        path: "tasks/motd/task.json",
        body:
            br#"{"files":{"/etc/motd":{"mode":"0600","group":"adm"},"/var/lib/motd":{"directory":true,"mode":"0750"}}}"#,
        mode: 0o644,
    },
    File::link("tasks/motd/tree/etc/mtab", b"/proc/self/mounts"),
];
pub const LINKS_MOVED: &[File] = &[
    File {
        path: "tasks/motd/task.json",
        body:
            br#"{"files":{"/etc/motd":{"mode":"0600","group":"adm"},"/var/lib/motd":{"directory":true,"mode":"0755"}}}"#,
        mode: 0o644,
    },
    File::link("tasks/motd/tree/etc/mtab", b"/proc/mounts"),
];
#[must_use]
pub fn diff_links() -> Rendered {
    let repository = Repository::make("links-diff", &super::thin_with(LINKS));
    let from = repository.head();
    let to = repository.next(LINKS_MOVED, &[]);
    super::diff(&repository, &super::Asked::new(&from, &to))
}
