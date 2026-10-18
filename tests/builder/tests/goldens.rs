use hearth_builder_world::fixtures;
use hearth_render_world::golden;
#[test]
fn merge_renders_what_its_expected_files_and_witnesses_say() {
    fixtures::check_golden("merge", "", &[]);
}
#[test]
fn merge_files_show_each_layers_value_in_its_witness() {
    let golden = golden::load("merge");
    let verdict = golden.witnesses(&fixtures::golden_output("merge"));
    assert!(verdict.is_clean(), "{}", verdict.report());
}
#[test]
fn merge_removal_by_null_leaves_the_task_out_of_an_env() {
    let targets = golden::Golden::targets(&fixtures::golden_output("merge"));
    let gone: Vec<_> =
        targets.iter().filter(|(_, text)| text.contains("/etc/gone ")).map(|(name, _)| name.as_str()).collect();
    assert_eq!(gone, ["m-b", "m-c"]);
}

#[test]
fn templates_render_byte_for_byte_as_their_witnesses_say() {
    fixtures::check_golden("templates", "", &[]);
}
const SYNTAX: &[(&str, &str)] = &[
    ("condition", "{% if "),
    ("loop", "{% for word in "),
    ("loop with pairs", "{% for name, number in "),
    ("assignment", "{% set "),
    ("raw", "{% raw %}"),
    ("default", "| default("),
    ("join", "| join("),
    ("split", "| split("),
    ("first", "| first"),
    ("last", "| last"),
    ("length", "| length"),
    ("replace", "| replace("),
    ("items", "| items"),
    ("keys", "| keys"),
    ("values", "| values"),
    ("sort", "| sort"),
    ("tojson", "| tojson"),
    ("shell_quote", "| shell_quote"),
    ("systemd_escape", "| systemd_escape "),
    ("systemd_escape_path", "| systemd_escape_path"),
    ("basename", "| basename"),
    ("dirname", "| dirname"),
];
#[test]
fn templates_cover_every_statement_and_filter_format_md_names() {
    let golden = golden::load("templates");
    let source = golden
        .tree
        .iter()
        .filter(|leaf| leaf.path.ends_with(b".j2"))
        .map(|leaf| std::str::from_utf8(&leaf.bytes).expect("template text"))
        .collect::<String>();
    for &(name, needle) in SYNTAX {
        assert!(source.contains(needle), "missing {name}");
    }
}
#[test]
fn the_fleet_renders_its_six_targets_as_expected() {
    fixtures::check_golden("fleet", "", &[]);
}
const X1_FIELDS: &[&str] = &[
    "target ",
    "placement ",
    "spec ",
    "packages ",
    "  sources ",
    "  request ",
    "  variant ",
    "  preseed    locales/default_environment_locale ",
    "  initramfs ",
    "config ",
    "  tmpfiles   /var/log/journal ",
    "  /etc/hostname ",
    "  /etc/localtime ",
    "data ",
    "  journal ",
];
const X1_RENDER_LINES: &str = concat!(
    "target       x1-prod   base x1   environment prod\n",
    "placement    metal efi 7th-gen\n",
    "spec         ............\n",
    "packages     part ............   resolution ............ needed\n",
    "  sources    forky amd64 main contrib non-free-firmware 20261001T030405Z\n",
    "  request    btrfs-progs ca-certificates dbus efibootmgr firmware-intel-sound iwd linux-image-amd64 locales sway tiny-initramfs tzdata\n",
    "  variant    apt\n",
    "  preseed    locales/default_environment_locale select en_US.UTF-8\n",
    "  initramfs  btrfs nvme\n",
    "config       part ............\n",
    "  tmpfiles   /var/log/journal d 2755 root systemd-journal\n",
    "  /etc/hostname                file 0644 root:root ............\n",
    "  /etc/localtime               link root:root /usr/share/zoneinfo/Europe/Lisbon\n",
    "data         part ............\n",
    "  journal    /var/log/journal\n",
);
#[test]
fn x1_prod_prints_render_md_9_2s_lines() {
    let text = include_bytes!("../../render/fleet/expected/x1-prod");
    assert_eq!(golden::mask12(&fixtures::lines_starting(text, X1_FIELDS)), X1_RENDER_LINES);
}
#[test]
fn a_lab_spec_does_not_change_when_a_production_file_does() {
    let repository = fixtures::golden_repository("fleet");
    let input = fixtures::render(&repository, &[b"carbon-lab"], hearth::Form::Text);
    drop(repository.next(&[fixtures::CARBON_PRODUCTION_EDIT], &[]));
    let after = fixtures::render(&repository, &[b"carbon-lab"], hearth::Form::Text);
    assert_eq!(after.stdout, input.stdout);
}
#[test]
fn the_container_has_no_hearth_task_and_its_host_has_one() {
    let repository = fixtures::golden_repository("fleet");
    drop(repository.next(&[], &["tasks/hearth/task.json"]));
    let rendered = fixtures::render(&repository, &[b"carbon-prod", b"chat-prod"], hearth::Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["target ", "tasks/"]),
        "target       chat-prod   base chat   environment prod\n\
tasks/hearth/task.json: carbon-prod: no task hearth under tasks/\n"
    );
}
#[test]
fn the_cluster_renders_its_members_with_their_names() {
    fixtures::check_golden("cluster", "", &[]);
}
#[test]
fn a_base_env_file_removes_a_member_from_its_environment() {
    let targets = golden::Golden::targets(&fixtures::golden_output("cluster"));
    assert_eq!(
        targets.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>(),
        ["stoa-1-lab", "stoa-1-prod", "stoa-2-lab", "stoa-2-prod", "stoa-3-lab"]
    );
}

#[test]
fn the_vpn_and_mail_golden_render_with_their_peers_per_environment() {
    fixtures::check_golden("exports", "", &[]);
}
#[test]
fn the_exports_golden_holds_hearth_20s_vpn_files() {
    let golden = golden::load("exports");
    for file in fixtures::VPN {
        let leaf = golden.tree.iter().find(|leaf| leaf.path == file.path.as_bytes()).expect(file.path);
        assert_eq!(leaf.bytes, file.body, "{}", file.path);
    }
}
#[test]
fn each_error_case_prints_its_errors_after_the_healthy_spec() {
    for name in fixtures::ERROR_CASES {
        fixtures::check_golden(name, "", &[]);
    }
}
#[test]
fn an_error_case_exits_2() {
    let repository = fixtures::golden_repository("errors/schema");
    let rendered = fixtures::render(&repository, &[], hearth::Form::Text);
    assert_eq!(fixtures::exit_code(&rendered), Some(2));
}
#[test]
fn the_tool_and_resolution_errors_print_through_the_protocol_world() {
    for (name, call, _) in fixtures::PROTOCOL_ERRORS {
        fixtures::protocol_golden(name, *call).check();
    }
}
#[test]
fn protocol_error_cases_exit_with_their_cause_class() {
    for (name, call, code) in fixtures::PROTOCOL_ERRORS {
        assert_eq!(fixtures::protocol_golden(name, *call).run.code, *code, "{name}");
    }
}

#[test]
fn errors_are_sorted_by_path_line_column_and_target() {
    let text = include_bytes!("../../render/errors/sorted/expected/output");
    assert_eq!(
        fixtures::lines_starting(text, &["tasks/"]),
        concat!(
            "tasks/another/task.json:2:15: bad-prod: another: requires a: first file\n",
            "tasks/broken/task.json:3:3: bad-prod: broken: requires a: first guard\n",
            "tasks/broken/task.json:3:3: bad2-prod: broken: requires a: first guard\n",
            "tasks/broken/task.json:3:48: bad-prod: broken: requires b: second guard\n",
            "tasks/broken/task.json:3:48: bad2-prod: broken: requires b: second guard\n",
        )
    );
}

#[test]
fn every_kind_of_7_2_but_checks_and_store_has_a_case() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../render/errors");
    let mut names: Vec<_> = std::fs::read_dir(root)
        .expect("error golden directory")
        .map(|entry| entry.expect("error golden").file_name().into_string().expect("kind name"))
        .filter(|name| name != "sorted")
        .collect();
    names.sort();
    assert_eq!(names, fixtures::ERROR_KINDS);
}
