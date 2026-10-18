//! Source inputs for the decoder sweep inventory.
use std::{
    fs,
    path::{Path, PathBuf},
};
#[must_use]
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
#[must_use]
pub fn source(world: &str, file: &str) -> String {
    fs::read_to_string(root().join("tests").join(world).join("tests").join(file)).expect("seeded sweep source")
}
#[must_use]
pub fn fuzzy_files() -> Vec<(String, String)> {
    let mut files = Vec::new();
    for world in fs::read_dir(root().join("tests")).expect("world directories") {
        let world = world.expect("world");
        let directory = world.path().join("tests");
        if !directory.is_dir() {
            continue;
        }
        for file in fs::read_dir(directory).expect("world tests") {
            let name = file.expect("test file").file_name().into_string().expect("UTF-8 file name");
            if name.starts_with("fuzzy_") && Path::new(&name).extension().is_some_and(|extension| extension == "rs") {
                files.push((world.file_name().into_string().expect("world name"), name));
            }
        }
    }
    files.sort();
    files
}
#[must_use]
pub fn machine_crates() -> Vec<String> {
    let mut crates = Vec::new();
    for entry in fs::read_dir(root().join("crates")).expect("crate directories") {
        let name = entry.expect("crate directory").file_name().into_string().expect("crate name");
        if let Some(name) = name.strip_prefix("hearth-protocol-") {
            crates.push(name.to_owned());
        }
    }
    crates.sort();
    crates
}
