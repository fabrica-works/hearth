//! Reserved rules cross intact (render.md, 7.2).
use crate::{records::errors as target, render::errors as source};
pub(super) fn reserved(value: source::Reserved) -> target::Reserved {
    target::Reserved { rule: rule(value.rule), what: value.what }
}
fn rule(rule: source::Rule) -> target::Rule {
    match rule {
        source::Rule::ActivatorKey => target::Rule::ActivatorKey,
        source::Rule::RootKey => target::Rule::RootKey,
        source::Rule::RootWait => target::Rule::RootWait,
        source::Rule::CmdlineLength { bytes } => target::Rule::CmdlineLength { bytes },
        source::Rule::PasswordPreseed => target::Rule::PasswordPreseed,
        source::Rule::TrialName => target::Rule::TrialName,
        source::Rule::Overlay => target::Rule::Overlay,
        source::Rule::TmpfilesSubvolume => target::Rule::TmpfilesSubvolume,
        source::Rule::TmpfilesWrite => target::Rule::TmpfilesWrite,
        source::Rule::TmpfilesPath => target::Rule::TmpfilesPath,
        source::Rule::ContainerModule => target::Rule::ContainerModule,
        source::Rule::ContainerParam => target::Rule::ContainerParam,
        source::Rule::ActivatorFile => target::Rule::ActivatorFile,
        source::Rule::Dropin => target::Rule::Dropin,
        source::Rule::Accounts => target::Rule::Accounts,
        source::Rule::SecretPath => target::Rule::SecretPath,
        source::Rule::HearthState => target::Rule::HearthState,
        source::Rule::PackagesLayer => target::Rule::PackagesLayer,
        source::Rule::ClassMount => target::Rule::ClassMount,
        source::Rule::DataRoot => target::Rule::DataRoot,
        source::Rule::Runtime => target::Rule::Runtime,
        source::Rule::LogFile => target::Rule::LogFile,
        source::Rule::InitramfsConf => target::Rule::InitramfsConf,
        source::Rule::SecretMode => target::Rule::SecretMode,
        source::Rule::HearthOwner => target::Rule::HearthOwner,
        source::Rule::LiveNone => target::Rule::LiveNone,
        source::Rule::NeverLiveUnit => target::Rule::NeverLiveUnit,
        source::Rule::NeverLivePath => target::Rule::NeverLivePath,
    }
}
