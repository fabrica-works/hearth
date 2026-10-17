use super::{File, Rendered, Repository};
pub const DATA: File = File { path: "tasks/motd/task.json", mode: 0o644, body: br#"{"data":{"journal":{"class":"core","path":"/var/log/journal","mode":"2755","group":"systemd-journal"},"iwd":{"class":"core","path":"/var/lib/iwd","mode":"0700"}},"groups":{"systemd-journal":{"from":"package","gid":999}}}"# };
pub const DATA_MOVED: File = File { path: "tasks/motd/task.json", mode: 0o644, body: br#"{"data":{"journal":{"class":"core","path":"/var/log/journal","mode":"2755","group":"systemd-journal"},"iwd":{"class":"data","path":"/var/lib/iwd","mode":"0700"}},"groups":{"systemd-journal":{"from":"package","gid":999}}}"# };
#[must_use]
pub fn diff_data() -> Rendered {
    let repository = Repository::make("data-diff", &super::thin_with(&[DATA]));
    let from = repository.head();
    let to = repository.next(&[DATA_MOVED], &[]);
    super::diff(&repository, &super::Asked::new(&from, &to))
}
