use super::{Asked, File, Rendered, Repository};
pub const SUBORDINATE: &[File] = &[
    File { path: "tasks/ci/task.json", mode: 0o644, body: br#"{"users": {"ci": {"subordinate": {"base": 100000, "count": 65536}}, "free": {"uid": 1000, "subordinate": {"base": 165536, "count": 65536}}}}"# },
    File { path: "hosts/x1/x1.json", mode: 0o644, body: br#"{"motd": {}, "ci": {}}"# },
];
pub const DATA_MODE: File = File { path: "tasks/motd/task.json", mode: 0o644, body: br#"{"data":{"journal":{"class":"core","path":"/var/log/journal","mode":"0755","group":"systemd-journal"},"iwd":{"class":"core","path":"/var/lib/iwd","mode":"0700"}},"groups":{"systemd-journal":{"from":"package","gid":999}}}"# };
pub const INITRAMFS_SHA256_HEX: &str = "6cb7c9647dfa51a50f2cb619e3644f84e0f908044fb1aeb9187232a98528caa8";
#[must_use]
pub fn preseeded() -> Repository {
    Repository::make("initramfs-cache", &super::thin_with(super::PRESEEDED))
}
#[must_use]
pub fn mode_diff(form: hearth::Form) -> Rendered {
    let repository = Repository::make("data-mode-diff", &super::thin_with(&[super::data::DATA]));
    let from = repository.head();
    let to = repository.next(&[DATA_MODE], &[]);
    super::diff(&repository, &Asked { form, ..Asked::new(&from, &to) })
}

#[must_use]
pub fn cached_initramfs() -> (Repository, std::path::PathBuf) {
    let repository = preseeded();
    drop(super::render(&repository, &[], hearth::Form::Text));
    let path = repository.scratch().path().join("store/cache").join(INITRAMFS_SHA256_HEX);
    (repository, path)
}
