use crate::{
    errors::Rule,
    form::Form,
    tests::fixtures::{self, reserved as f},
};
const TEXT: &[(Rule, &[u8])] = &[
    (Rule::ActivatorKey, b"\"rootwait\" is reserved: the activator owns this command-line key (machine.md, 6.2)\n"),
    (Rule::RootKey, b"\"rootwait\" is reserved: it changes how tiny-initramfs finds or starts the root (machine.md, 6.2)\n"),
    (Rule::RootWait, b"\"rootwait\" is reserved: tiny-initramfs would wait for ever for a missing root (machine.md, 6.2)\n"),
    (Rule::CmdlineLength { bytes: f::CMDLINE_BYTES }, b"\"rootwait\" is reserved: the command line is 2049 bytes, past 2048 with the activator's tokens and NUL (machine.md, 6.2)\n"),
    (Rule::PasswordPreseed, b"\"rootwait\" is reserved: a password preseed leaves its value in /var/cache/debconf/passwords.dat (format.md, 5.11)\n"),
    (Rule::TrialName, b"\"rootwait\" is reserved: hearth owns trial and check units (slots.md, 5.5)\n"),
    (Rule::Overlay, b"\"rootwait\" is reserved: extensions would merge over hearth's live overlays (format.md, 5.11)\n"),
    (Rule::TmpfilesSubvolume, b"\"rootwait\" is reserved: a nested subvolume breaks a layer (format.md, 5.11)\n"),
    (Rule::TmpfilesWrite, b"\"rootwait\" is reserved: a tmpfiles write outside /var, /run or /tmp would fail on the read-only root (format.md, 5.11)\n"),
    (Rule::TmpfilesPath, b"\"rootwait\" is reserved: a tmpfiles path holds no glob or specifier (format.md, 5.11)\n"),
    (Rule::ContainerModule, b"\"rootwait\" is reserved: a container has no initramfs module (render.md, 4.2)\n"),
    (Rule::ContainerParam, b"\"rootwait\" is reserved: a container has no boot parameter (render.md, 4.2)\n"),
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
];
#[test]
fn json_gives_the_rule_and_what() {
    for (row, &(rule, expected)) in JSON.iter().enumerate() {
        assert_eq!(fixtures::print_error(Form::JsonLines, f::error(rule)).as_deref(), Ok(expected), "row {row}");
    }
}
