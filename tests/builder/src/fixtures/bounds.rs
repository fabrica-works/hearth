//! Peaks shared by the service and protocol layer's checked worlds.
use hearth_builder_world::fixtures::{self, File, Scenario};
// The smallest service queue holds every MAX_OUT. One more target forces
// a later pass while printer output is still moving through that queue.
pub const QUEUE: u32 = 3;
pub const TARGETS: u32 = QUEUE + 1;
pub const SEED: u64 = fixtures::SIMULATION_SEED;
#[must_use]
pub fn limits() -> hearth::Limits {
    let mut limits = fixtures::small_limits();
    limits.queue = QUEUE;
    limits.domain.render.targets = TARGETS;
    limits
}
#[must_use]
pub fn queued() -> Scenario<'static> {
    let mut files = fixtures::THIN.to_vec();
    for target in 0..TARGETS {
        files.push(File {
            path: format!("hosts/b{target}/b{target}.json").leak(),
            body: b"{\"motd\":{}}",
            mode: 0o644,
        });
        files.push(File {
            path: format!("hosts/b{target}/b{target}-prod.json").leak(),
            body: fixtures::reserved::METAL_EFI,
            mode: 0o644,
        });
    }
    files.retain(|file| !file.path.starts_with("hosts/x1/"));
    Scenario { files: files.leak(), ..fixtures::thin() }
}
