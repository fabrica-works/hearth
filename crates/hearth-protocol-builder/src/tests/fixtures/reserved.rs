use crate::render;
#[must_use]
pub fn composition_kinds() -> [render::errors::Composition; 10] {
    use crate::render::errors::{Composition, Host};
    [
        Composition::Duplicate,
        Composition::Long,
        Composition::Placement,
        Composition::Members,
        Composition::Unplaced,
        Composition::Null,
        Composition::Task,
        Composition::On { on: Box::from(b"host-lab".as_slice()), host: Host::Unknown },
        Composition::On { on: Box::from(b"host-lab".as_slice()), host: Host::Container },
        Composition::On {
            on: Box::from(b"host-lab".as_slice()),
            host: Host::Elsewhere { environment: Box::from(b"lab".as_slice()) },
        },
    ]
}
pub const CMDLINE_BYTES: u32 = 2049; // One past machine.md, 6.2's bound.
pub const RESERVED_RULES: &[render::errors::Rule] = &[
    render::errors::Rule::ActivatorKey,
    render::errors::Rule::RootKey,
    render::errors::Rule::RootWait,
    render::errors::Rule::CmdlineLength { bytes: CMDLINE_BYTES },
    render::errors::Rule::PasswordPreseed,
    render::errors::Rule::TrialName,
    render::errors::Rule::Overlay,
    render::errors::Rule::TmpfilesSubvolume,
    render::errors::Rule::TmpfilesWrite,
    render::errors::Rule::TmpfilesPath,
    render::errors::Rule::ContainerModule,
    render::errors::Rule::ContainerParam,
    render::errors::Rule::ActivatorFile,
    render::errors::Rule::Dropin,
    render::errors::Rule::Accounts,
    render::errors::Rule::SecretPath,
    render::errors::Rule::HearthState,
    render::errors::Rule::PackagesLayer,
    render::errors::Rule::ClassMount,
    render::errors::Rule::DataRoot,
    render::errors::Rule::Runtime,
    render::errors::Rule::LogFile,
    render::errors::Rule::InitramfsConf,
    render::errors::Rule::SecretMode,
    render::errors::Rule::HearthOwner,
    render::errors::Rule::LiveNone,
    render::errors::Rule::NeverLiveUnit,
    render::errors::Rule::NeverLivePath,
];
#[must_use]
pub fn reserved_rules() -> &'static [render::errors::Rule] {
    RESERVED_RULES
}
