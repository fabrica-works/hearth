use hearth::Form;
use hearth_builder_world::fixtures::{
    self,
    part_changes::{self, Part},
};
use skein_io::kernel::Exit;
#[test]
fn hearth_diff_names_a_download_moved_and_rebuilds_from_software() {
    let rendered = part_changes::changed(Part::Software, Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  software "]),
        "  software ~ /opt/ripgrep                           tar 1 root:root d2571cab3997\n"
    );
}
#[test]
fn hearth_diff_names_a_secret_moved_and_prepares_a_new_slot() {
    let rendered = part_changes::changed(Part::Secrets, Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  secrets "]),
        "  secrets  ~ secret mail/tls/mx1.key\n  secrets  ~ password free\n"
    );
}
#[test]
fn hearth_diff_a_live_path_added_rebuilds_nothing() {
    let rendered = part_changes::changed(Part::Live, Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  live "]),
        "  live     + /etc/nginx/                            motd nginx.service reload\n"
    );
}
#[test]
fn hearth_diff_seeds_alone_rebuild_from_packages() {
    let rendered = part_changes::changed(Part::Seeds, Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["x1-prod", "  packages "]),
        concat!(
            "x1-prod                                             rebuild from packages, resolution needed\n",
            "  packages   resolution same\n",
            "  packages ~ seed group opendkim                    121\n"
        )
    );
}
const REBUILDS: &[(Part, &str)] = &[
    (Part::Software, r#""rebuild":{"kind":"from_software"}"#),
    (Part::Secrets, r#""rebuild":{"kind":"slot_only"}"#),
    (Part::Live, r#""rebuild":{"kind":"nothing"}"#),
    (Part::Seeds, r#""rebuild":{"kind":"from_packages","resolution":"needed"}"#),
];
#[test]
fn changed_parts_report_their_rebuild_kind_in_json() {
    for (part, expected) in REBUILDS {
        let rendered = part_changes::changed(*part, Form::JsonLines);
        let stdout = std::str::from_utf8(&rendered.stdout).expect("JSON");
        assert!(stdout.contains(expected), "{part:?}: {stdout}");
    }
}
#[test]
fn each_changed_part_exits_with_changes() {
    for part in part_changes::PARTS {
        assert_eq!(part_changes::changed(*part, Form::Text).exit, Exit::Code(1), "{part:?}");
    }
}
