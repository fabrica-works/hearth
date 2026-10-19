use hearth_builder_world::fixtures::{
    self,
    reserved::{self, hearth as f},
};
use skein_io::kernel::Exit;
#[test]
fn the_hearth_task_passes_every_reserved_rule() {
    let rendered = f::render_hearth();
    assert_eq!(
        fixtures::lines_starting(
            &rendered.stdout,
            &["  request ", "  users ", "  members ", "  presets ", "  download ", "  fetch "]
        ),
        "  request    git mmdebstrap openssh-client uidmap\n  download   /usr/local/bin/hearth file 0755 root:root ca2dff161724\n  fetch      ca2dff161724 https://git.ekanayaka.io/ai/hearth/releases/download/v0.1.0/hearth-x86_64-linux\n  users      hearth 59999 hearth /nonexistent /usr/sbin/nologin\n  members    hearth systemd-journal\n  presets    hearth-activator.service enable\n  presets    systemd-confext.service disable\n  presets    systemd-sysext.service disable\n"
    );
}
#[test]
fn the_hearth_task_renders_successfully() {
    assert_eq!(f::render_hearth().exit, Exit::Code(0));
}
#[test]
fn an_empty_hearth_task_requests_no_packages() {
    assert_eq!(fixtures::lines_starting(&f::render_empty_hearth().stdout, &["  request"]), "");
}
#[test]
fn an_empty_hearth_task_renders_successfully() {
    assert_eq!(f::render_empty_hearth().exit, Exit::Code(0));
}
#[test]
fn a_container_renders_without_the_hearth_declaration() {
    let rendered = f::render_no_hearth(b"chat-prod");
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["target ", "tasks/", "hosts/"]),
        "target       chat-prod   base chat   environment prod\n"
    );
}
#[test]
fn a_container_without_hearth_exits_zero() {
    assert_eq!(f::render_no_hearth(b"chat-prod").exit, Exit::Code(0));
}
#[test]
fn a_host_names_the_missing_hearth_declaration() {
    assert_eq!(
        f::render_no_hearth(b"rhi-prod").stdout,
        b"tasks/hearth/task.json: rhi-prod: no task hearth under tasks/\n"
    );
}
#[test]
fn a_host_without_hearth_exits_two() {
    assert_eq!(f::render_no_hearth(b"rhi-prod").exit, reserved::CONFIGURATION_EXIT);
}
#[test]
fn a_container_names_hearth_at_its_key() {
    assert_eq!(f::render_container_hearth().stdout, b"hosts/chat/chat-prod.json:1:56: chat-prod: hearth: \"hearth\" is reserved: a container has no hearth task (protocol/format.md, 5.12)\n");
}
#[test]
fn a_container_naming_hearth_exits_two() {
    assert_eq!(f::render_container_hearth().exit, reserved::CONFIGURATION_EXIT);
}
