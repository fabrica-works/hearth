use crate::{records::errors as source, render::errors as target, tests::fixtures};
fn back(rule: source::Rule) -> target::Rule {
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
    }
}
#[test]
fn each_rule_translates_to_the_records_and_back() {
    for (row, rule) in fixtures::reserved_rules().iter().copied().enumerate() {
        let input = target::Reserved { rule, what: Box::from(b"rootwait".as_slice()) };
        let mut error = fixtures::decode_error();
        error.kind = target::Kind::Reserved(input.clone());
        let source::Kind::Reserved(reserved) = crate::translate::errors::error(error).kind else { panic!("reserved") };
        let back = target::Reserved { rule: back(reserved.rule), what: reserved.what };
        assert_eq!(back, input, "row {row}: reserved rule and word");
    }
}
