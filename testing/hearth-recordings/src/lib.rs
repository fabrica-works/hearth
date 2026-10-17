//! Captured tool runs, testing.md, 4.5. Paths are relative to this checkout.
use std::{
    fs,
    path::{Path, PathBuf},
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recording {
    pub version: Box<str>,
    pub argv: Vec<Vec<u8>>,
    pub env: Vec<Vec<u8>>,
    pub dir: Vec<u8>,
    pub stdin: Option<Vec<u8>>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit: Box<str>,
    pub expect: Option<Vec<u8>>,
    pub lists: Option<Vec<u8>>,
    pub stanzas: Option<Vec<u8>>,
}
fn root(tool: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../recordings").join(tool)
}
fn lines(bytes: &[u8]) -> Vec<Vec<u8>> {
    if bytes.is_empty() {
        return Vec::new();
    }
    bytes.strip_suffix(b"\n").unwrap_or(bytes).split(|b| *b == b'\n').map(<[u8]>::to_vec).collect()
}
fn joined(lines: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for line in lines {
        bytes.extend_from_slice(line);
        bytes.push(b'\n');
    }
    bytes
}
#[must_use]
pub fn read(tool: &str, version: &str, case: &str) -> Recording {
    read_at(&root(tool).join(version).join(case), version)
}
fn read_at(dir: &Path, version: &str) -> Recording {
    let get = |name| {
        fs::read(dir.join(name)).unwrap_or_else(|e| panic!("{}: {e}; capture with HEARTH_RECORD=1", dir.display()))
    };
    let optional = |name| match fs::read(dir.join(name)) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => panic!("{}: {error}", dir.display()),
    };
    Recording {
        version: version.into(),
        argv: lines(&get("argv")),
        env: lines(&get("env")),
        dir: get("dir"),
        stdin: optional("stdin"),
        stdout: get("stdout"),
        stderr: get("stderr"),
        exit: String::from_utf8(get("exit")).expect("exit text").trim().into(),
        expect: optional("expect"),
        lists: optional("lists"),
        stanzas: optional("stanzas"),
    }
}
#[must_use]
pub fn all(tool: &str) -> Vec<(Box<str>, Recording)> {
    let mut result = Vec::new();
    let mut versions = directories(&root(tool));
    versions.sort();
    for version in versions {
        let mut cases = directories(&version);
        cases.sort();
        for case in cases {
            let name = case.file_name().expect("case name").to_str().expect("case UTF-8").into();
            result.push((name, read_at(&case, version.file_name().expect("version").to_str().expect("version UTF-8"))));
        }
    }
    result
}
fn directories(at: &Path) -> Vec<PathBuf> {
    fs::read_dir(at)
        .unwrap_or_else(|e| panic!("{}: {e}; capture with HEARTH_RECORD=1", at.display()))
        .map(|e| e.expect("directory entry").path())
        .filter(|p| p.is_dir())
        .collect()
}
pub fn write(tool: &str, version: &str, case: &str, recording: &Recording) {
    write_at(&root(tool).join(version).join(case), recording);
}
fn write_at(dir: &Path, recording: &Recording) {
    match fs::remove_dir_all(dir) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("{}: {error}", dir.display()),
    }
    fs::create_dir_all(dir).expect("recording directory");
    for (name, bytes) in [
        ("argv", joined(&recording.argv)),
        ("env", joined(&recording.env)),
        ("dir", recording.dir.clone()),
        ("stdout", recording.stdout.clone()),
        ("stderr", recording.stderr.clone()),
        ("exit", format!("{}\n", recording.exit).into_bytes()),
    ] {
        fs::write(dir.join(name), bytes).expect("recording file");
    }
    for (name, bytes) in [
        ("stdin", &recording.stdin),
        ("expect", &recording.expect),
        ("lists", &recording.lists),
        ("stanzas", &recording.stanzas),
    ] {
        if let Some(bytes) = bytes {
            fs::write(dir.join(name), bytes).expect("optional recording file");
        }
    }
}
#[derive(Debug)]
pub struct Mask {
    scratch: Vec<u8>,
    further: Vec<(Vec<u8>, Vec<u8>)>,
}
impl Mask {
    #[must_use]
    pub fn new(scratch: &Path) -> Self {
        Self { scratch: scratch.as_os_str().as_encoded_bytes().to_vec(), further: Vec::new() }
    }
    /// Mask another literal after the scratch path.
    #[must_use]
    pub fn also(mut self, from: &[u8], to: &[u8]) -> Self {
        assert!(!from.is_empty(), "a mask literal is nonempty");
        self.further.push((from.to_vec(), to.to_vec()));
        self
    }
    #[must_use]
    pub fn bytes(&self, bytes: &[u8]) -> Vec<u8> {
        let mut result = Vec::new();
        let mut at = 0;
        while at < bytes.len() {
            if bytes[at..].starts_with(&self.scratch) {
                result.extend_from_slice(b"/scratch");
                at += self.scratch.len();
            } else if bytes[at..].starts_with(&self.scratch[1..]) {
                result.extend_from_slice(b"scratch");
                at += self.scratch.len() - 1;
            } else {
                result.push(bytes[at]);
                at += 1;
            }
        }
        for (from, to) in &self.further {
            let mut replaced = Vec::new();
            let mut at = 0;
            while at < result.len() {
                if result[at..].starts_with(from) {
                    replaced.extend_from_slice(to);
                    at += from.len();
                } else {
                    replaced.push(result[at]);
                    at += 1;
                }
            }
            result = replaced;
        }
        result
    }
    pub fn recording(&self, recording: &mut Recording) {
        for bytes in recording.argv.iter_mut().chain(recording.env.iter_mut()) {
            *bytes = self.bytes(bytes);
        }
        for bytes in [&mut recording.dir, &mut recording.stdout, &mut recording.stderr] {
            *bytes = self.bytes(bytes);
        }
        for bytes in [&mut recording.stdin, &mut recording.expect, &mut recording.lists, &mut recording.stanzas]
            .into_iter()
            .flatten()
        {
            *bytes = self.bytes(bytes);
        }
    }
}
#[must_use]
pub fn recording_on() -> bool {
    std::env::var_os("HEARTH_RECORD").is_some_and(|v| v == "1")
}
#[cfg(test)]
mod tests;
