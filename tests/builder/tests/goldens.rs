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
