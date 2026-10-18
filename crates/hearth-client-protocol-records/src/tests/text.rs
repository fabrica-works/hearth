use super::fixtures;
use crate::{
    form::{self, Form, Refusal},
    limits,
    outcome::Outcome,
    parts::config::{Node, TreeEntry},
    records::{Record, Terminal, Type},
};
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
#[test]
fn a_spec_prints_its_meta_and_config_as_render_md_shows() {
    let printed = form::print(Form::Text, &Record::Spec(fixtures::x1_prod()), None, &limits::SHIPPED);
    assert_eq!(printed.as_deref(), Ok(fixtures::X1_PROD_TEXT.as_bytes()), "render.md, 9.2's lines");
}
#[test]
fn a_digest_prints_its_first_twelve_lowercase_hex_digits() {
    let mut spec = fixtures::x1_prod();
    spec.digest = fixtures::digest([0xab, 0xcd, 0xef, 0x01, 0x23, 0x45]);
    let printed = fixtures::packages_text(spec);
    assert!(skein_lib::bytes::find(&printed, b"spec         abcdef012345\n").is_some(), "twelve lowercase digits");
}
#[test]
fn a_mode_prints_as_four_octal_digits() {
    for &(mode, expected) in MODES {
        let mut spec = fixtures::x1_prod();
        let config = fixtures::config_mut(&mut spec);
        let Node::File { mode: value, .. } = &mut fixtures::only_mut(&mut config.tree).node else {
            panic!("file fixture")
        };
        *value = mode;
        let printed = fixtures::packages_text(spec);
        assert!(skein_lib::bytes::find(&printed, expected).is_some(), "four octal digits for {mode:o}");
    }
}
#[test]
fn a_path_past_the_column_widens_it_for_every_entry() {
    let mut spec = fixtures::x1_prod();
    let config = fixtures::config_mut(&mut spec);
    let node = fixtures::only_mut(&mut config.tree).node.clone();
    config.tree = Box::new([
        TreeEntry { path: copy_of(b"/a"), node: node.clone() },
        TreeEntry { path: copy_of(b"/a/very/long/path/past/column"), node },
    ]);
    let printed = fixtures::packages_text(spec);
    let expected = concat!(
        "target       x1-prod   base x1   environment prod\n",
        "placement    metal efi 7th-gen\n",
        "spec         9a0c5e1f2b3d\n",
        "config       part e7c2a0d95b13\n",
        "  /a                             file 0644 root:root 3f1c2ab07e44\n",
        "  /a/very/long/path/past/column  file 0644 root:root 3f1c2ab07e44\n"
    );
    assert_eq!(&*printed, expected.as_bytes(), "every entry shares the widened column");
}
#[test]
fn a_second_spec_follows_a_blank_line() {
    let printed = form::print(Form::Text, &Record::Spec(fixtures::x1_prod()), Some(Type::Spec), &limits::SHIPPED)
        .expect("valid spec");
    assert_eq!(printed.first(), Some(&b'\n'), "a blank line before another spec");
    assert_eq!(&printed[1..], fixtures::X1_PROD_TEXT.as_bytes(), "the spec follows");
}
#[test]
fn the_terminal_record_of_a_render_prints_nothing() {
    let printed = form::print(
        Form::Text,
        &Record::Terminal(Terminal { outcome: Outcome::Done, diff: None, left_out: 0 }),
        Some(Type::Spec),
        &limits::SHIPPED,
    );
    assert_eq!(printed.as_deref(), Ok(&b""[..]), "render has no text terminal line");
}
#[test]
fn a_record_past_its_limit_is_refused_too_long() {
    let spec = Record::Spec(fixtures::x1_prod());
    assert_eq!(
        form::print(Form::Text, &spec, None, &limits::Limits { record: 64, ..limits::SHIPPED }),
        Err(Refusal::TooLong),
        "refused before allocation"
    );
    assert_eq!(
        form::print(Form::Text, &spec, None, &limits::SHIPPED).as_deref(),
        Ok(fixtures::X1_PROD_TEXT.as_bytes()),
        "shipped admission"
    );
}

#[test]
fn a_known_resolution_prints_the_keys_line_last() {
    let spec = fixtures::with_resolution(fixtures::x1_prod(), fixtures::known());
    let printed = fixtures::packages_text(spec);
    let expected = concat!(
        "target       x1-prod   base x1   environment prod\n",
        "placement    metal efi 7th-gen\n",
        "spec         9a0c5e1f2b3d\n",
        "config       part e7c2a0d95b13\n",
        "  /etc/hostname            file 0644 root:root 3f1c2ab07e44\n",
        "keys         packages 111111111111   software 222222222222   config 333333333333\n",
    );
    assert_eq!(&*printed, expected.as_bytes(), "known layer keys are the last line");
}
#[test]
fn a_needed_resolution_prints_no_keys_line() {
    let spec = fixtures::with_resolution(
        fixtures::x1_prod(),
        crate::records::Resolution::Needed { key: fixtures::digest([0x0c, 0x7d, 0x9e, 0x41, 0xc0, 0xd2]) },
    );
    let printed = form::print(Form::Text, &Record::Spec(spec), None, &limits::SHIPPED);
    assert_eq!(printed.as_deref(), Ok(fixtures::X1_PROD_TEXT.as_bytes()), "needed resolution adds no keys line");
}

#[test]
fn a_link_and_a_directory_print_after_the_files_columns() {
    assert_eq!(
        form::print(Form::Text, &Record::Spec(fixtures::links_and_directories()), None, &limits::SHIPPED)
            .expect("valid spec")
            .as_ref(),
        concat!(
            "target       x1-prod   base x1   environment prod\n",
            "placement    metal efi 7th-gen\n",
            "spec         9a0c5e1f2b3d\n",
            "config       part e7c2a0d95b13\n",
            "  /etc/mtab                link root:root /proc/self/mounts\n",
            "  /var/lib/motd            directory 0750 root:adm\n"
        )
        .as_bytes()
    );
}
const MODES: &[(u16, &[u8])] = &[(0o644, b"0644"), (0o4755, b"4755")];
