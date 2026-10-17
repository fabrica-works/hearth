//! Declaration maps (format.md, 5.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Map {
    Packages,
    Preseeds,
    Initramfs,
    Software,
    Files,
    Users,
    Groups,
    Presets,
    Tmpfiles,
    Data,
    Secrets,
    Identity,
    Cmdline,
    Live,
}
