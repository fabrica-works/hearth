use super::{File, Form, Rendered, Repository, THIN, fs, render, thin_with};
pub const EXPORTS: &[File] = &[
    File { path: "tasks/vpn/task.json", mode: 0o644, body: b"{\"settings\": {\"address\": {\"type\": \"string\", \"doc\": \"Address.\"}, \"endpoint\": {\"type\": \"string\", \"optional\": true, \"doc\": \"Endpoint.\"}},\n \"exports\": {\"address\": {\"type\": \"string\", \"value\": \"{{ settings.address }}\", \"doc\": \"Address.\"}, \"endpoint\": {\"type\": \"string\", \"optional\": true, \"value\": \"{{ settings.endpoint }}\", \"doc\": \"Endpoint.\"}}}" },
    File { path: "hosts/x1/x1.json", mode: 0o644, body: br#"{"motd": {}, "vpn": {"address": "10.0.0.2"}}"# },
    File { path: "hosts/x1/x1-prod.json", mode: 0o644, body: br#"{"placement":{"kind":"metal","firmware":"efi","hardware":"7th-gen"},"vpn":{"endpoint":"198.51.100.7"}}"# },
];
pub const DEEP_EXPORT: u32 = 29;
pub const DEEP_NEST_JSON: &[u8] = br#""nest":[[[[[[[[[[[[[[[[[[[[[[[[[[[[[1]]]]]]]]]]]]]]]]]]]]]]]]]]]]]"#;
#[must_use]
pub fn published(form: Form) -> Rendered {
    let repository = Repository::make("exports", &thin_with(EXPORTS));
    render(&repository, &[b"x1-prod"], form)
}
#[must_use]
pub fn export_value(value: &str, form: Form) -> Rendered {
    let repository = Repository::make("export-value", &thin_with(EXPORTS));
    let declaration = format!(
        "{{\"settings\": {{\"address\": {{\"type\": \"string\", \"doc\": \"Address.\"}}, \"endpoint\": {{\"type\": \"string\", \"optional\": true, \"doc\": \"Endpoint.\"}}}},\n \"exports\": {{\"address\": {{\"type\": \"string\", \"value\": \"{value}\", \"doc\": \"Address.\"}}}}}}"
    );
    fs::write(repository.work_tree().join("tasks/vpn/task.json"), declaration).expect("export declaration");
    drop(repository.next(&[], &[]));
    render(&repository, &[b"x1-prod"], form)
}
#[must_use]
pub fn optional_export() -> Rendered {
    let repository = Repository::make("export-optional", &thin_with(EXPORTS));
    drop(repository.next(&[thin_placement()], &[]));
    render(&repository, &[b"x1-prod"], Form::Text)
}
#[must_use]
pub fn deep(depth: u32) -> Repository {
    let repository = Repository::make("deep-export", &thin_with(EXPORTS));
    let mut ty = "\"int\"".to_owned();
    let mut value = "1".to_owned();
    for _ in 0..depth {
        ty = format!("{{\"list\":{ty}}}");
        value = format!("[{value}]");
    }
    let declaration =
        format!("{{\"exports\":{{\"nest\":{{\"type\":{ty},\"value\":\"{{{{ {value} }}}}\",\"doc\":\"Nest.\"}}}}}}");
    fs::write(repository.work_tree().join("tasks/vpn/task.json"), declaration).expect("deep declaration");
    drop(repository.next(
        &[File { path: "hosts/x1/x1.json", body: br#"{"motd":{},"vpn":{}}"#, mode: 0o644 }, thin_placement()],
        &[],
    ));
    repository
}

fn thin_placement() -> File {
    *THIN.iter().find(|file| file.path == "hosts/x1/x1-prod.json").expect("thin placement")
}
