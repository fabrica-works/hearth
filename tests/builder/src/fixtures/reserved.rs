use super::{File, Form, Rendered, Repository};
pub const WAIT: File =
    File { path: "tasks/motd/task.json", body: br#"{"cmdline": {"wait": {"param": "rootwait"}}}"#, mode: 0o644 };
pub const PASSWORD: File = File {
    path: "tasks/motd/task.json",
    body: br#"{"preseeds": {"d/p": {"type": "password", "value": "x"}}}"#,
    mode: 0o644,
};
pub const PARAMETER_BYTES: usize = 1000; // Two parameters make 139 + 2 * (1 + 1000) = 2141 bytes.
#[must_use]
pub fn long_params() -> String {
    format!(
        "{{\"cmdline\": {{\"a\": {{\"param\": \"{}\"}}, \"b\": {{\"param\": \"{}\"}}}}}}",
        "a".repeat(PARAMETER_BYTES),
        "b".repeat(PARAMETER_BYTES)
    )
}
#[must_use]
pub fn rendered(file: &File) -> Rendered {
    let repository = Repository::make("reserved-word", &super::thin_with(core::slice::from_ref(file)));
    super::render(&repository, &[], Form::Text)
}
#[must_use]
pub fn long_line() -> Rendered {
    let repository = Repository::make("reserved-length", super::THIN);
    std::fs::write(repository.work_tree().join("tasks/motd/task.json"), long_params()).expect("long parameters");
    drop(repository.git(&["add", "-A"]));
    drop(repository.git(&["commit", "-q", "-m", "parameters"]));
    super::render(&repository, &[], Form::Text)
}
