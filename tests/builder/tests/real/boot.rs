use hearth::Form;
use hearth_builder_world::fixtures::{self, Repository};
use skein_io::kernel::Exit;
#[test]
fn the_boot_part_lists_the_parameters_in_order() {
    let repository = fixtures::command_line_repository("cmdline", false);
    assert_eq!(
        fixtures::lines_starting(
            &fixtures::render(&repository, &[], Form::Text).stdout,
            &["boot ", "  params", "  mutable"]
        ),
        "boot         part 68ef6aa14d25\n  params     quiet audit=0 console=ttyS0,115200n8\n  mutable    false\n"
    );
}
#[test]
fn an_order_past_999_prints_its_fault_at_the_field() {
    let repository = fixtures::command_line_repository("cmdline-order", true);
    assert_eq!(
        fixtures::render(&repository, &[], Form::Text).stdout,
        b"tasks/params/task.json:1:41: x1-prod: params: past its max 999\n"
    );
}
#[test]
fn an_order_past_999_exits_with_configuration_failure() {
    let repository = fixtures::command_line_repository("cmdline-order-exit", true);
    assert_eq!(fixtures::render(&repository, &[], Form::Text).exit, Exit::Code(2));
}
#[test]
fn placement_mutable_true_prints_mutable_true() {
    let placement = fixtures::File {
        path: "hosts/x1/x1-prod.json",
        body: br#"{"placement": {"kind": "metal", "firmware": "efi", "hardware": "7th-gen", "mutable": true}}"#,
        mode: 0o644,
    };
    let repository = Repository::make("mutable", &fixtures::thin_with(core::slice::from_ref(&placement)));
    assert_eq!(
        fixtures::lines_starting(&fixtures::render(&repository, &[], Form::Text).stdout, &["  mutable"]),
        "  mutable    true\n"
    );
}
