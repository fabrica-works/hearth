use super::fixtures;
use crate::{
    form::{self, Form},
    limits,
};
#[test]
fn presets_tmpfiles_and_scripts_print_as_choices_9_shows() {
    let text = form::print(Form::Text, &fixtures::directives_spec(), None, &limits::SHIPPED).expect("settings");
    assert_eq!(text.as_ref(), concat!(
        "target       x1-prod   base x1   environment prod\nplacement    metal efi 7th-gen\nspec         9a0c5e1f2b3d\nconfig       part 000000000000\n",
        "  presets    getty@tty1.service disable\n  presets    ssh.service enable\n",
        "  tmpfiles   /var/log/journal d 2755 root systemd-journal\n  scripts    motd 300 010101010101\n"
    ).as_bytes());
}
#[test]
fn config_settings_keep_their_types_and_values_in_json() {
    let json = form::print(Form::JsonLines, &fixtures::directives_spec(), None, &limits::SHIPPED).expect("settings");
    let start =
        skein_lib::bytes::find(&json, br#"{"record":"part","target":"x1-prod","name":"config""#).expect("config");
    assert_eq!(json.get(start..).expect("suffix"), concat!(r#"{"record":"part","target":"x1-prod","name":"config","digest":"0000000000000000000000000000000000000000000000000000000000000000","config":{"groups":{},"members":[],"presets":{"getty@tty1.service":{"kind":"disable"},"ssh.service":{"kind":"enable"}},"scripts":[{"content_digest":"0101010101010101010101010101010101010101010101010101010101010101","order":300,"task":"motd"}],"tmpfiles":{"/var/log/journal":{"group":"systemd-journal","mode":"2755","owner":"root","type":"d"}},"tree":{},"users":{}}}"#, "\n").as_bytes());
}
#[test]
fn settings_changes_pad_their_keys_to_the_diff_column() {
    let text = form::print(Form::Text, &fixtures::directives_change(), None, &limits::SHIPPED).expect("changes");
    assert_eq!(
        text.as_ref(),
        concat!(
            "x1-prod                                             rebuild from config\n",
            "  config   ~ presets ssh.service                    disable\n",
            "  config   ~ tmpfiles /var/log/journal              d 0755 root systemd-journal\n",
            "  config   ~ scripts                                motd 300 020202020202\n",
        )
        .as_bytes()
    );
}
#[test]
fn settings_changes_keep_both_values_in_json() {
    let json = form::print(
        Form::JsonLines,
        &fixtures::directives_change(),
        Some(crate::records::Type::Change),
        &limits::SHIPPED,
    )
    .expect("changes");
    assert_eq!(json.as_ref(), concat!(r#"{"record":"change","target":"x1-prod","kind":"changed","changed":{"parts":[{"kind":"config","groups":[],"users":[],"members":[],"presets":[{"after":{"kind":"disable"},"before":{"kind":"enable"},"key":"ssh.service","kind":"changed"}],"scripts":{"after":[{"content_digest":"0202020202020202020202020202020202020202020202020202020202020202","order":300,"task":"motd"}],"before":[{"content_digest":"0101010101010101010101010101010101010101010101010101010101010101","order":300,"task":"motd"}]},"tmpfiles":[{"after":{"group":"systemd-journal","mode":"0755","owner":"root","type":"d"},"before":{"group":"systemd-journal","mode":"2755","owner":"root","type":"d"},"key":"/var/log/journal","kind":"changed"}],"tree":[]}],"rebuild":{"kind":"from_config"}}}"#, "\n").as_bytes());
}
