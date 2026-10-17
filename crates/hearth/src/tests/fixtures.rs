use std::ffi::OsString;
pub(super) fn words(input: &[&str]) -> Vec<OsString> {
    input.iter().map(OsString::from).collect()
}
