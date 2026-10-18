use hearth_render_world::golden::{self, Golden};
use std::ffi::OsStr;
const BLESS: &[(Option<&str>, bool)] =
    &[(Some("1"), true), (None, false), (Some("0"), false), (Some(""), false), (Some("11"), false)];
#[test]
fn bless_is_asked_by_the_variable_set_to_1_alone() {
    for &(value, expected) in BLESS {
        assert_eq!(golden::asked(value.map(OsStr::new)), expected, "{value:?}");
    }
}
const TARGETS: &[(&str, &[(&str, &str)])] = &[
    (
        "target a\nspec 111111111111\ntarget b\nspec 222222222222\n",
        &[("a", "target a\nspec 111111111111\n"), ("b", "target b\nspec 222222222222\n")],
    ),
    ("target c\nspec 333333333333\n", &[("c", "target c\nspec 333333333333\n")]),
];
#[test]
fn a_spec_text_is_cut_into_its_targets_at_each_target_line() {
    for &(text, expected) in TARGETS {
        assert_eq!(
            Golden::targets(text).iter().map(|(name, text)| (name.as_str(), text.as_str())).collect::<Vec<_>>(),
            expected
        );
    }
}
const WITNESSES: &[(&str, bool)] = &[
    ("  /etc/empty file 0644 root:root e3b0c44298fc", true),
    ("  /etc/empty file 0644 root:root e3b0c44298f", false),
    ("  /etc/empty file 0644 root:root 0e3b0c44298fc", false),
    ("  /etc/other file 0644 root:root e3b0c44298fc", false),
];
#[test]
fn a_witness_must_match_its_files_line_to_twelve_digits() {
    for &(line, expected) in WITNESSES {
        assert_eq!(golden::witness_line(line, "/etc/empty", "e3b0c44298fc"), expected, "{line}");
    }
}
#[test]
fn a_golden_without_an_expected_file_is_a_failure_naming_it() {
    let verdict = golden::load("merge").check("missing", "target m-a\nspec 111111111111\n", Some(0));
    assert!(verdict.report().contains("merge/expected-missing/m-a missing"), "{}", verdict.report());
}
const LINKS: &[(&str, bool)] = &[
    ("  /etc/localtime          link root:root /zone/with space", true),
    ("  /etc/localtime          link root:root /zone/with", false),
    ("  /etc/other              link root:root /zone/with space", false),
    ("  /etc/localtime          file 0644 root:root /zone/with space", false),
];
#[test]
fn a_link_witness_matches_its_target_text() {
    for &(line, expected) in LINKS {
        assert_eq!(golden::link_witness_line(line, "/etc/localtime", "/zone/with space"), expected, "{line}");
    }
}
