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
    let printed = form::print(Form::Text, &Record::Spec(spec), None, &limits::SHIPPED).expect("valid spec");
    assert!(skein_lib::bytes::find(&printed, b"spec         abcdef012345\n").is_some(), "twelve lowercase digits");
}
#[test]
fn a_mode_prints_as_four_octal_digits() {
    for (mode, expected) in [(0o644, b"0644"), (0o4755, b"4755")] {
        let mut spec = fixtures::x1_prod();
        let crate::records::PartRecord::Config(config) = &mut spec.parts[0].record;
        let Node::File { mode: value, .. } = &mut config.tree[0].node;
        *value = mode;
        let printed = form::print(Form::Text, &Record::Spec(spec), None, &limits::SHIPPED).expect("valid spec");
        assert!(skein_lib::bytes::find(&printed, expected).is_some(), "four octal digits for {mode:o}");
    }
}
#[test]
fn a_path_past_the_column_widens_it_for_every_entry() {
    let mut spec = fixtures::x1_prod();
    let crate::records::PartRecord::Config(config) = &mut spec.parts[0].record;
    let node = config.tree[0].node.clone();
    config.tree = Box::new([
        TreeEntry { path: copy_of(b"/a"), node: node.clone() },
        TreeEntry { path: copy_of(b"/a/very/long/path/past/column"), node },
    ]);
    let printed = form::print(Form::Text, &Record::Spec(spec), None, &limits::SHIPPED).expect("valid spec");
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
        &Record::Terminal(Terminal { outcome: Outcome::Done }),
        Some(Type::Spec),
        &limits::SHIPPED,
    );
    assert_eq!(printed.as_deref(), Ok(&b""[..]), "render has no text terminal line");
}
#[test]
fn a_record_past_its_limit_is_refused_too_long() {
    let spec = Record::Spec(fixtures::x1_prod());
    assert_eq!(
        form::print(Form::Text, &spec, None, &limits::Limits { record: 64 }),
        Err(Refusal::TooLong),
        "refused before allocation"
    );
    assert_eq!(
        form::print(Form::Text, &spec, None, &limits::SHIPPED).as_deref(),
        Ok(fixtures::X1_PROD_TEXT.as_bytes()),
        "shipped admission"
    );
}
