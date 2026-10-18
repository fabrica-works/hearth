use hearth::Form;
use hearth_builder_world::fixtures;
use skein_io::kernel::Exit;
#[test]
fn two_recorded_resolutions_show_the_packages_a_change_moves() {
    let rendered = fixtures::recorded_resolution_diff("recorded-diff", Form::JsonLines);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["{\"record\":\"change\""]),
        concat!(
            r#"{"record":"change","target":"test-min","kind":"changed","changed":{"parts":[{"kind":"packages","packages":{"request":{"added":["vim"],"removed":[]},"seeds":{"groups":[],"users":[]},"resolution":{"added":[{"architecture":"amd64","name":"vim","version":"9"}],"kind":"resolved","removed":[{"architecture":"amd64","name":"nano","version":"8"}],"versions_changed":[{"after":"3","architecture":"amd64","before":"2","name":"apt"}]}}}],"rebuild":{"kind":"from_packages","resolution":"known"}}}"#,
            "\n"
        )
    );
}
#[test]
fn two_recorded_resolutions_exit_with_a_negative_diff() {
    assert_eq!(fixtures::recorded_resolution_diff("recorded-diff-exit", Form::JsonLines).exit, Exit::Code(1));
}
#[test]
fn recorded_resolution_changes_print_each_package_in_text() {
    let rendered = fixtures::recorded_resolution_diff("recorded-diff-text", Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  packages", "    "]),
        concat!(
            "  packages   resolution new key, resolved\n",
            "    + vim:amd64 9\n",
            "    - nano:amd64 8\n",
            "    ~ apt:amd64 2 → 3\n",
            "  packages + vim\n"
        )
    );
}
