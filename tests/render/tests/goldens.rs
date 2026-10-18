use hearth_render_world::fixtures;
#[test]
fn merge_digests_do_not_depend_on_arrival_order() {
    fixtures::goldens::check_digests("merge");
}

#[test]
fn templates_digests_do_not_depend_on_arrival_order() {
    fixtures::goldens::check_digests("templates");
}
const REPOSITORIES: &[&str] = &["fleet", "cluster"];
#[test]
fn fleet_and_cluster_digests_do_not_depend_on_arrival_order() {
    for name in REPOSITORIES {
        fixtures::goldens::check_digests(name);
    }
}
