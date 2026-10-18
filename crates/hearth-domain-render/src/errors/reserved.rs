//! Closed reserved rules (format.md, 5.11; machine.md, 6.2).
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Reserved {
    pub rule: Rule,
    pub what: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Rule {
    ActivatorKey,
    RootKey,
    RootWait,
    CmdlineLength { bytes: u32 },
    PasswordPreseed,
    TrialName,
    Overlay,
    TmpfilesSubvolume,
    TmpfilesWrite,
    TmpfilesPath,
    ContainerModule,
    ContainerParam,
}
