use hearth_render_world::fixtures;
#[test]
fn merge_digests_do_not_depend_on_arrival_order() {
    fixtures::goldens::check_digests("merge");
}

#[test]
fn templates_digests_do_not_depend_on_arrival_order() {
    fixtures::goldens::check_digests("templates");
}
