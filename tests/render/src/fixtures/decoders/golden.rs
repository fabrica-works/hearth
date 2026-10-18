//! The reviewed repositories as decoder corpora, shared by the two parser worlds.
use std::{
    fs,
    path::{Path, PathBuf},
};
/// Four schedules per reviewed input exercise the original and its mutations.
pub const CASES: core::ops::Range<u64> = 0..4;
pub const SEED: u64 = 0x0030_6000;
const REPOSITORIES: &[&str] = &["merge", "templates", "fleet", "cluster", "exports", "errors"];
/// Read source files only, excluding expectations, rendered witnesses and links.
#[must_use]
pub fn corpus(extension: &str) -> Vec<(PathBuf, Vec<u8>)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../render");
    let mut files = Vec::new();
    for name in REPOSITORIES {
        visit(&root.join(name), extension, &mut files);
    }
    assert!(!files.is_empty(), "reviewed {extension} corpus exists");
    files
}
fn visit(directory: &Path, extension: &str, files: &mut Vec<(PathBuf, Vec<u8>)>) {
    let mut entries: Vec<_> =
        fs::read_dir(directory).expect("golden directory").map(|entry| entry.expect("golden entry")).collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let name = entry.file_name();
        if name.to_string_lossy().starts_with("expected") || name == "rendered" || name == ".git" {
            continue;
        }
        let path = entry.path();
        let kind = entry.file_type().expect("golden kind");
        if kind.is_dir() {
            visit(&path, extension, files);
        } else if kind.is_file() && path.extension().is_some_and(|value| value == extension) {
            files.push((path.clone(), fs::read(&path).expect("golden source")));
        }
    }
}
