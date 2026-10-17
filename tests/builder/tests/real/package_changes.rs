use hearth_builder_world::fixtures;
use skein_io::kernel::Exit;
#[test]
fn hearth_diff_lists_an_added_package_and_rebuilds_from_packages() {
    let rendered = fixtures::packages_diff("packages-diff");
    let text = String::from_utf8(rendered.stdout).expect("JSON lines");
    assert!(text.contains("\"added\":[\"vim\"],\"removed\":[]"), "{text}");
    assert!(text.contains("\"kind\":\"from_packages\""), "{text}");
}
#[test]
fn an_added_package_exits_with_a_negative_diff() {
    assert_eq!(fixtures::packages_diff("packages-diff-exit").exit, Exit::Code(1));
}
#[test]
fn a_changed_command_line_is_a_new_slot_of_the_same_output() {
    let rendered = fixtures::boot_diff("boot-diff");
    let text = String::from_utf8(rendered.stdout).expect("JSON lines");
    assert!(text.contains("\"kind\":\"slot_only\""), "{text}");
}
#[test]
fn a_changed_command_line_exits_with_a_negative_diff() {
    assert_eq!(fixtures::boot_diff("boot-diff-exit").exit, Exit::Code(1));
}
