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

pub const FILES: &[File] = &[
    File { path: "tasks/motd/task.json", mode: 0o644,
        body: br#"{"files":{"/etc/motd.d/{{ value }}":{"for_each":"['dev', 'ops']","template":"templates/greeting.j2","mode":"0640"},"/etc/ssl/lab.crt":{"copy":"lab-ca/lab.crt"}}}"# },
    File { path: "tasks/motd/templates/greeting.j2", mode: 0o644, body: b"Hello {{ value }} on {{ target.name }}.\n" },
    File { path: "lab-ca/lab.crt", mode: 0o644, body: b"lab certificate\n" },
];
pub const FILES_HI: &[File] =
    &[File { path: "tasks/motd/templates/greeting.j2", mode: 0o644, body: b"Hi {{ value }} on {{ target.name }}.\n" }];
#[must_use]
pub fn diff_files() -> Rendered {
    let repository = Repository::make("files-diff", &super::thin_with(FILES));
    let from = repository.head();
    let to = repository.next(FILES_HI, &[]);
    super::diff(&repository, &super::Asked::new(&from, &to))
}
