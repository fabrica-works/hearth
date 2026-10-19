use std::{collections::BTreeMap, fs, path::Path};

/// `testing/mirror/pin`, read (domain/testing.md, 6.2); a bump is a commit of it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Pin {
    pub archive: Box<str>,
    pub suite: Box<str>,
    pub architecture: Box<str>,
    /// `main` first; M1's is `main` alone.
    pub components: Box<[Box<str>]>,
    /// `InRelease`'s SHA-256, 64 lowercase hex digits.
    pub digest: Box<str>,
    pub targets: Box<[Target]>,
}
/// A test repository's target and its strictly sorted request.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Target {
    pub name: Box<str>,
    pub variant: Box<str>,
    pub request: Box<[Box<str>]>,
}
/// A refused pin's line from 1 (0 for a missing field), and its fault.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PinError {
    pub line: u32,
    pub wrong: Wrong,
}
/// Unknown field, repeated field, missing field, or malformed field value.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Wrong {
    Unknown,
    Twice,
    Missing(&'static str),
    Form(&'static str),
}
impl Pin {
    pub fn read(text: &[u8]) -> Result<Self, PinError> {
        let mut fields = BTreeMap::new();
        let mut targets = Vec::new();
        for (index, bytes) in text.split(|byte| *byte == b'\n').enumerate() {
            let line = u32::try_from(index + 1).expect("the pin's line count fits u32");
            let fail = |wrong| PinError { line, wrong };
            let text = std::str::from_utf8(bytes).map_err(|_| fail(Wrong::Form("text")))?;
            let words: Vec<&str> = text.split('#').next().unwrap_or("").split_whitespace().collect();
            let Some(&field) = words.first() else { continue };
            let values = &words[1..];
            if field == "target" {
                if values.len() < 2 || values[2..].windows(2).any(|pair| pair[0] >= pair[1]) {
                    return Err(fail(Wrong::Form("target")));
                }
                if targets.iter().any(|target: &Target| &*target.name == values[0]) {
                    return Err(fail(Wrong::Twice));
                }
                targets.push(Target {
                    name: values[0].into(),
                    variant: values[1].into(),
                    request: values[2..].iter().map(|v| (*v).into()).collect(),
                });
                continue;
            }
            let valid = match field {
                "archive" => values.len() == 1 && timestamp(values[0]),
                "digest" => {
                    values.len() == 1
                        && values[0].len() == 64
                        && values[0].bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                }
                "components" => values.first() == Some(&"main"),
                "suite" | "architecture" => values.len() == 1,
                _ => return Err(fail(Wrong::Unknown)),
            };
            if fields.contains_key(field) {
                return Err(fail(Wrong::Twice));
            }
            if !valid {
                let name = match field {
                    "archive" => "archive",
                    "digest" => "digest",
                    "components" => "components",
                    "suite" => "suite",
                    "architecture" => "architecture",
                    _ => unreachable!("known field"),
                };
                return Err(fail(Wrong::Form(name)));
            }
            fields.insert(field, values.to_vec());
        }
        for field in ["archive", "suite", "architecture", "components", "digest"] {
            if !fields.contains_key(field) {
                return Err(PinError { line: 0, wrong: Wrong::Missing(field) });
            }
        }
        Ok(Self {
            archive: fields["archive"][0].into(),
            suite: fields["suite"][0].into(),
            architecture: fields["architecture"][0].into(),
            components: fields["components"].iter().map(|v| (*v).into()).collect(),
            digest: fields["digest"][0].into(),
            targets: targets.into_boxed_slice(),
        })
    }
    /// Reads the committed pin beside this crate; panics naming a bad pin.
    #[must_use]
    pub fn committed() -> Self {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../mirror/pin");
        let bytes = fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        Self::read(&bytes).unwrap_or_else(|error| panic!("{}: {error:?}", path.display()))
    }
    #[must_use]
    pub fn target(&self, name: &str) -> Option<&Target> {
        self.targets.iter().find(|target| &*target.name == name)
    }
}
fn timestamp(text: &str) -> bool {
    text.len() == 16
        && text.bytes().enumerate().all(|(i, b)| match i {
            8 => b == b'T',
            15 => b == b'Z',
            _ => b.is_ascii_digit(),
        })
}
