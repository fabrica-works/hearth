use super::{Exit, Form, fixtures};
#[test]
fn hearth_render_prints_the_exports_a_target_publishes() {
    assert_eq!(
        fixtures::lines_starting(&fixtures::published(Form::Text).stdout, &["  vpn."]),
        "  vpn.address    \"10.0.0.2\"\n  vpn.endpoint   \"198.51.100.7\"\n"
    );
}
#[test]
fn published_exports_leave_the_render_successful() {
    assert_eq!(fixtures::published(Form::Text).exit, Exit::Code(0));
}
#[test]
fn an_optional_export_with_no_setting_prints_no_line() {
    assert_eq!(
        fixtures::lines_starting(&fixtures::optional_export().stdout, &["  vpn."]),
        "  vpn.address   \"10.0.0.2\"\n"
    );
}
#[test]
fn json_prints_the_exports_part_nested_under_its_task() {
    assert!(
        skein_lib::bytes::find(
            &fixtures::published(Form::JsonLines).stdout,
            b"\"exports\":{\"vpn\":{\"address\":\"10.0.0.2\",\"endpoint\":\"198.51.100.7\"}}"
        )
        .is_some()
    );
}
#[test]
fn an_export_nested_29_lists_deep_prints_as_json() {
    let repository = fixtures::deep(fixtures::DEEP_EXPORT);
    let rendered = fixtures::render(&repository, &[b"x1-prod"], Form::JsonLines);
    assert!(
        skein_lib::bytes::find(&rendered.stdout, fixtures::DEEP_NEST_JSON).is_some(),
        "{}",
        String::from_utf8_lossy(&rendered.stdout)
    );
}
#[test]
fn an_export_of_the_wrong_type_prints_at_its_value() {
    assert_eq!(
        fixtures::lines_starting(&fixtures::export_value("{{ 7 }}", Form::Text).stdout, &["tasks/vpn/"]),
        "tasks/vpn/task.json:2:53: x1-prod: vpn: not a string\n"
    );
}
#[test]
fn a_value_with_text_around_its_tag_prints_once_without_a_target() {
    assert_eq!(
        fixtures::lines_starting(
            &fixtures::export_value("v {{ settings.address }}", Form::Text).stdout,
            &["tasks/vpn/"]
        ),
        "tasks/vpn/task.json:2:53: vpn: an export's value must be one {{ }} tag\n"
    );
}
#[test]
fn an_export_naming_an_undeclared_setting_prints_at_its_name() {
    assert_eq!(
        fixtures::lines_starting(&fixtures::export_value("{{ settings.adress }}", Form::Text).stdout, &["tasks/vpn/"]),
        "tasks/vpn/task.json:2:53: vpn: undeclared setting \"adress\"; vpn declares address, endpoint\n"
    );
}
#[test]
fn an_export_reading_tasks_is_refused() {
    assert_eq!(
        fixtures::lines_starting(
            &fixtures::export_value("{{ tasks.core.domain }}", Form::Text).stdout,
            &["tasks/vpn/"]
        ),
        "tasks/vpn/task.json:2:53: vpn: an export reads settings and target alone, not tasks\n"
    );
}
#[test]
fn a_reader_sees_the_exports_of_its_own_environment_alone() {
    assert_eq!(
        fixtures::lines_starting(&fixtures::vpn(false).stdout, &["  /etc/vpn/peers"]),
        concat!(
            "  /etc/vpn/peers           file 0644 root:root afd16377136a\n",
            "  /etc/vpn/peers           file 0644 root:root 0ed834effddb\n",
        )
    );
}
#[test]
fn environment_exports_leave_the_render_successful() {
    assert_eq!(fixtures::vpn(false).exit, Exit::Code(0));
}
#[test]
fn an_exporters_fault_prints_for_it_and_for_its_reader() {
    assert_eq!(
        fixtures::lines_starting(&fixtures::vpn(true).stdout, &["hosts/ion/ion-prod.json:"]),
        concat!(
            "hosts/ion/ion-prod.json:1:92: carbon-prod: vpn: exports.vpn of ion-prod: not a string\n",
            "hosts/ion/ion-prod.json:1:92: ion-prod: vpn: not a string\n",
        )
    );
}
#[test]
fn an_exporters_fault_leaves_the_other_environment_rendering() {
    assert_eq!(
        fixtures::lines_starting(&fixtures::vpn(true).stdout, &["target       "]),
        concat!(
            "target       carbon-lab   base carbon   environment lab\n",
            "target       ion-lab   base ion   environment lab\n",
        )
    );
}
#[test]
fn an_exporters_fault_makes_the_render_fail() {
    assert_eq!(fixtures::vpn(true).exit, Exit::Code(2));
}
