use crate::{
    errors::Rule,
    form::Form,
    tests::fixtures::{self, reserved as f},
};
const TEXT: &[(Rule, &[u8])] = &[
    (Rule::ActivatorKey, b"\"rootwait\" is reserved: the activator owns this command-line key (domain/machine.md, 6.2)\n"),
    (Rule::RootKey, b"\"rootwait\" is reserved: it changes how tiny-initramfs finds or starts the root (domain/machine.md, 6.2)\n"),
    (Rule::RootWait, b"\"rootwait\" is reserved: tiny-initramfs would wait for ever for a missing root (domain/machine.md, 6.2)\n"),
    (Rule::CmdlineLength { bytes: f::CMDLINE_BYTES }, b"\"rootwait\" is reserved: the command line is 2049 bytes, past 2048 with the activator's tokens and NUL (domain/machine.md, 6.2)\n"),
    (Rule::PasswordPreseed, b"\"rootwait\" is reserved: a password preseed leaves its value in /var/cache/debconf/passwords.dat (protocol/format.md, 5.11)\n"),
    (Rule::TrialName, b"\"rootwait\" is reserved: hearth owns trial and check units (domain/slots.md, 5.5)\n"),
    (Rule::Overlay, b"\"rootwait\" is reserved: extensions would merge over hearth's live overlays (protocol/format.md, 5.11)\n"),
    (Rule::TmpfilesSubvolume, b"\"rootwait\" is reserved: a nested subvolume breaks a layer (protocol/format.md, 5.11)\n"),
    (Rule::TmpfilesWrite, b"\"rootwait\" is reserved: a tmpfiles write outside /var, /run or /tmp would fail on the read-only root (protocol/format.md, 5.11)\n"),
    (Rule::TmpfilesPath, b"\"rootwait\" is reserved: a tmpfiles path holds no glob or specifier (protocol/format.md, 5.11)\n"),
    (Rule::ContainerModule, b"\"rootwait\" is reserved: a container has no initramfs module (domain/render.md, 4.2)\n"),
    (Rule::ContainerParam, b"\"rootwait\" is reserved: a container has no boot parameter (domain/render.md, 4.2)\n"),
    (Rule::ActivatorFile, b"\"rootwait\" is reserved: the activator writes this path at prepare (protocol/format.md, 5.11)\n"),
    (Rule::Dropin, b"\"rootwait\" is reserved: the watchdog's drop-in must sort last (domain/slots.md, 5.5)\n"),
    (Rule::Accounts, b"\"rootwait\" is reserved: accounts and machine identity are written at prepare (protocol/format.md, 5.11)\n"),
    (Rule::SecretPath, b"rootwait is a secret's path; declare it under secrets (protocol/format.md, 9.1)\n"),
    (Rule::HearthState, b"\"rootwait\" is reserved: hearth owns its state and store (domain/machine.md, 3.4)\n"),
    (Rule::PackagesLayer, b"\"rootwait\" is reserved: the packages layer owns /usr outside /usr/local (protocol/format.md, 5.5)\n"),
    (Rule::ClassMount, b"\"rootwait\" is reserved: the class mount hides this path (protocol/format.md, 5.7)\n"),
    (Rule::DataRoot, b"\"rootwait\" is reserved: a data path must be strictly beneath /srv, /var or /home (protocol/format.md, 5.7)\n"),
    (Rule::Runtime, b"\"rootwait\" is reserved: runtime, noise and history are not task files (protocol/format.md, 5.11)\n"),
    (Rule::LogFile, b"\"rootwait\" is reserved: a file beneath /var/log must be empty (protocol/format.md, 5.11)\n"),
    (Rule::InitramfsConf, b"\"rootwait\" is reserved: the packages part writes tiny-initramfs configuration (domain/machine.md, 7.1)\n"),
    (Rule::SecretMode, b"\"rootwait\" is reserved: a secret or identity mode gives others nothing and has no special bits (protocol/format.md, 9.1)\n"),
    (Rule::HearthOwner, b"\"rootwait\" is reserved: hearth must not be able to read this secret or identity file (protocol/format.md, 9.1)\n"),
    (Rule::LiveNone, b"\"rootwait\" is reserved: action none names no unit (protocol/format.md, 5.14)\n"),
    (Rule::NeverLiveUnit, b"\"rootwait\" is reserved: this core service must not restart live (protocol/format.md, 5.14)\n"),
    (Rule::NeverLivePath, b"\"rootwait\" is reserved: this path must not change live (protocol/format.md, 5.14)\n"),
    (Rule::HearthContainer, b"\"rootwait\" is reserved: a container has no hearth task (protocol/format.md, 5.12)\n"),
    (Rule::HearthNull, b"\"rootwait\" is reserved: the hearth task cannot be removed (protocol/format.md, 5.12)\n"),
];
#[test]
fn a_reserved_error_prints_the_rules_words() {
    for (row, &(rule, expected)) in TEXT.iter().enumerate() {
        assert_eq!(fixtures::print_error(Form::Text, f::error(rule)).as_deref(), Ok(expected), "row {row}");
    }
}
const JSON: &[(Rule, &[u8])] = &[
    (Rule::RootWait, b"{\"record\":\"error\",\"kind\":\"reserved\",\"reserved\":{\"rule\":\"rootwait\",\"what\":\"rootwait\"},\"class\":\"configuration\"}\n"),
    (Rule::CmdlineLength { bytes: f::CMDLINE_BYTES }, b"{\"record\":\"error\",\"kind\":\"reserved\",\"reserved\":{\"bytes\":2049,\"rule\":\"cmdline_length\",\"what\":\"rootwait\"},\"class\":\"configuration\"}\n"),
    (Rule::ActivatorFile, b"{\"record\":\"error\",\"kind\":\"reserved\",\"reserved\":{\"rule\":\"activator_file\",\"what\":\"rootwait\"},\"class\":\"configuration\"}\n"),
    (Rule::Dropin, b"{\"record\":\"error\",\"kind\":\"reserved\",\"reserved\":{\"rule\":\"dropin\",\"what\":\"rootwait\"},\"class\":\"configuration\"}\n"),
    (Rule::Accounts, b"{\"record\":\"error\",\"kind\":\"reserved\",\"reserved\":{\"rule\":\"accounts\",\"what\":\"rootwait\"},\"class\":\"configuration\"}\n"),
    (Rule::SecretPath, b"{\"record\":\"error\",\"kind\":\"reserved\",\"reserved\":{\"rule\":\"secret_path\",\"what\":\"rootwait\"},\"class\":\"configuration\"}\n"),
    (Rule::HearthState, b"{\"record\":\"error\",\"kind\":\"reserved\",\"reserved\":{\"rule\":\"hearth_state\",\"what\":\"rootwait\"},\"class\":\"configuration\"}\n"),
    (Rule::PackagesLayer, b"{\"record\":\"error\",\"kind\":\"reserved\",\"reserved\":{\"rule\":\"packages_layer\",\"what\":\"rootwait\"},\"class\":\"configuration\"}\n"),
    (Rule::ClassMount, b"{\"record\":\"error\",\"kind\":\"reserved\",\"reserved\":{\"rule\":\"class_mount\",\"what\":\"rootwait\"},\"class\":\"configuration\"}\n"),
    (Rule::DataRoot, b"{\"record\":\"error\",\"kind\":\"reserved\",\"reserved\":{\"rule\":\"data_root\",\"what\":\"rootwait\"},\"class\":\"configuration\"}\n"),
    (Rule::Runtime, b"{\"record\":\"error\",\"kind\":\"reserved\",\"reserved\":{\"rule\":\"runtime\",\"what\":\"rootwait\"},\"class\":\"configuration\"}\n"),
    (Rule::LogFile, b"{\"record\":\"error\",\"kind\":\"reserved\",\"reserved\":{\"rule\":\"log_file\",\"what\":\"rootwait\"},\"class\":\"configuration\"}\n"),
    (Rule::InitramfsConf, b"{\"record\":\"error\",\"kind\":\"reserved\",\"reserved\":{\"rule\":\"initramfs_conf\",\"what\":\"rootwait\"},\"class\":\"configuration\"}\n"),
    (Rule::SecretMode, b"{\"record\":\"error\",\"kind\":\"reserved\",\"reserved\":{\"rule\":\"secret_mode\",\"what\":\"rootwait\"},\"class\":\"configuration\"}\n"),
    (Rule::HearthOwner, b"{\"record\":\"error\",\"kind\":\"reserved\",\"reserved\":{\"rule\":\"hearth_owner\",\"what\":\"rootwait\"},\"class\":\"configuration\"}\n"),
    (Rule::LiveNone, b"{\"record\":\"error\",\"kind\":\"reserved\",\"reserved\":{\"rule\":\"live_none\",\"what\":\"rootwait\"},\"class\":\"configuration\"}\n"),
    (Rule::NeverLiveUnit, b"{\"record\":\"error\",\"kind\":\"reserved\",\"reserved\":{\"rule\":\"never_live_unit\",\"what\":\"rootwait\"},\"class\":\"configuration\"}\n"),
    (Rule::NeverLivePath, b"{\"record\":\"error\",\"kind\":\"reserved\",\"reserved\":{\"rule\":\"never_live_path\",\"what\":\"rootwait\"},\"class\":\"configuration\"}\n"),
    (Rule::HearthContainer, b"{\"record\":\"error\",\"kind\":\"reserved\",\"reserved\":{\"rule\":\"hearth_container\",\"what\":\"rootwait\"},\"class\":\"configuration\"}\n"),
    (Rule::HearthNull, b"{\"record\":\"error\",\"kind\":\"reserved\",\"reserved\":{\"rule\":\"hearth_null\",\"what\":\"rootwait\"},\"class\":\"configuration\"}\n"),
];
#[test]
fn json_gives_the_rule_and_what() {
    for (row, &(rule, expected)) in JSON.iter().enumerate() {
        assert_eq!(fixtures::print_error(Form::JsonLines, f::error(rule)).as_deref(), Ok(expected), "row {row}");
    }
}
