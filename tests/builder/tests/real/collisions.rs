use hearth_builder_world::fixtures::{self, collisions};
use skein_io::kernel::Exit;
#[test]
fn hearth_render_names_both_tasks_of_a_path() {
    let rendered = collisions::render("tree-twice", collisions::TREE_TWICE);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["tasks/banner/"]),
        "tasks/banner/task.json:1:12: x1-prod: /etc/motd is given by banner and by motd (tasks/motd/tree/etc/motd)\n"
    );
}
#[test]
fn a_tree_collision_exits_with_a_configuration_error() {
    assert_eq!(collisions::render("tree-twice-exit", collisions::TREE_TWICE).exit, Exit::Code(2));
}
#[test]
fn hearth_render_refuses_a_file_under_a_data_path() {
    let rendered = collisions::render("hidden", collisions::HIDDEN);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["tasks/app/"]),
        "tasks/app/task.json:1:11: x1-prod: /var/lib/app/seed is under the data path /var/lib/app (a mount would hide it) (tasks/dump/tree/var/lib/app/seed)\n"
    );
}
#[test]
fn a_hidden_path_exits_with_a_configuration_error() {
    assert_eq!(collisions::render("hidden-exit", collisions::HIDDEN).exit, Exit::Code(2));
}
#[test]
fn hearth_render_names_the_two_tasks_of_a_differing_user() {
    let rendered = collisions::render("differing-user", collisions::CLASH);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["tasks/a/"]),
        "tasks/a/task.json:1:12: x1-prod: user \"free\" differs between a and b (tasks/b/task.json:1:12)\n"
    );
}
#[test]
fn hearth_render_names_the_two_users_sharing_a_uid() {
    let rendered = collisions::render("shared-uid", collisions::SHARED);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["tasks/a/"]),
        "tasks/a/task.json:1:12: x1-prod: uid 1000 is shared by free and ci (tasks/b/task.json:1:12)\n"
    );
}
#[test]
fn differing_users_exit_with_a_configuration_error() {
    assert_eq!(collisions::render("differing-user-exit", collisions::CLASH).exit, Exit::Code(2));
}
#[test]
fn a_shared_uid_exits_with_a_configuration_error() {
    assert_eq!(collisions::render("shared-uid-exit", collisions::SHARED).exit, Exit::Code(2));
}
