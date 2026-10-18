use super::{ELEMENT, bytes, optional, record, strings};
use crate::parts::packages::Packages;
pub(super) fn packages(packages: &Packages) -> u64 {
    let sources = &packages.sources;
    let mut components = 0_u64;
    for component in &sources.components {
        components = components.saturating_add(ELEMENT).saturating_add(bytes(component.name()));
    }
    let initramfs = match &packages.initramfs {
        Some(initramfs) => {
            record(&[(b"modules", strings(&initramfs.modules)), (b"microcode", bytes(initramfs.microcode.name()))])
        }
        None => 0,
    };
    let mut held = record(&[
        (b"initramfs_file", u64::from(packages.initramfs_file.is_some()).saturating_mul(32)),
        (
            b"sources",
            record(&[
                (b"suite", bytes(&sources.suite)),
                (b"architecture", bytes(&sources.architecture)),
                (b"components", components),
                (b"archive", bytes(&sources.archive)),
                (b"updates", 0),
                (b"security", optional(sources.security.as_deref())),
            ]),
        ),
        (b"request", strings(&packages.request)),
        (b"variant", bytes(packages.variant.name())),
        (b"preseeds", 0),
        (b"initramfs", initramfs),
        (b"apt", record(&[(b"install_recommends", 0)])),
        (b"seeds", record(&[(b"users", 0), (b"groups", 0)])),
    ]);
    for preseed in &packages.preseeds {
        held = held.saturating_add(ELEMENT).saturating_add(record(&[
            (b"question", bytes(&preseed.question)),
            (b"owner", bytes(&preseed.owner)),
            (b"kind", bytes(preseed.kind.name())),
            (b"value", bytes(&preseed.value)),
        ]));
    }
    for user in &packages.seeds.users {
        held = held.saturating_add(ELEMENT).saturating_add(record(&[
            (b"name", bytes(&user.name)),
            (b"uid", 0),
            (b"gid", 0),
        ]));
    }
    for group in &packages.seeds.groups {
        held = held.saturating_add(ELEMENT).saturating_add(record(&[(b"name", bytes(&group.name)), (b"gid", 0)]));
    }
    held
}
