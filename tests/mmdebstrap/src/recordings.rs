//! Real mmdebstrap captures paired with hand-reviewed expectations (domain/testing.md, 4.5).
use crate::reference::{self, Decoded};
use hearth_recordings::Recording;
use std::{fs, path::Path};
#[derive(Debug)]
pub struct Case {
    pub case: Box<str>,
    pub recording: Recording,
    pub expected: Option<Decoded>,
}
#[must_use]
pub fn all() -> Vec<Case> {
    hearth_recordings::all("mmdebstrap")
        .into_iter()
        .map(|(case, recording)| {
            let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("expect")
                .join(&*recording.version)
                .join(format!("{case}.expect"));
            let expected = match fs::read_to_string(&path) {
                Ok(text) => Some(reference::expectation(&text)),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => panic!("{}: {error}", path.display()),
            };
            Case { case, recording, expected }
        })
        .collect()
}
