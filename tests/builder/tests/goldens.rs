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
