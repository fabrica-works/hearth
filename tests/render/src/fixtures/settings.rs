//! A scaled composed-settings budget reaches every target in a cheap memory case.
use super::{Repository, entry, set_document, task_patch, with_setting};
use hearth_domain_render::{
    limits::{Limits, SHIPPED},
    revision::{Kind, declaration::Form},
};

// Thirty-two targets suffice to distinguish keeping one settings map from all.
const COMPOSED_BYTES: u32 = 65_536; // A KiB for each of the 64 shipped MiB.
pub const LIMITS: Limits = Limits {
    targets: 32,
    composed: COMPOSED_BYTES as u64,
    decoded: (COMPOSED_BYTES as u64) * 2,
    expansion: 32,
    ..SHIPPED
};
// Leave room for the enclosing task and setting keys in the composed map.
pub const SETTING_BYTES: usize = COMPOSED_BYTES as usize - 256;
pub const MEMORY_SEED: u64 = 0x28_53;
#[must_use]
pub fn every_target() -> Repository {
    let mut repository = with_setting(Form::String);
    set_document(&mut repository, b'1', task_patch(b"motd", b"x", Kind::String(vec![b'x'; SETTING_BYTES].into())));
    repository.entries.retain(|entry| !entry.path.starts_with(b"hosts/"));
    for index in 0..LIMITS.targets {
        // Equal host documents share their object; each target still composes
        // its own settings and placement from that document and the environment.
        repository.entries.push(entry(format!("hosts/t{index:04}/t{index:04}-prod.json").as_bytes(), b'3'));
    }
    repository.entries.sort_by(|first, second| first.path.cmp(&second.path));
    repository
}
