//! Reviewed repository goldens and independently written file witnesses (render.md, 9.2).
use std::{
    ffi::OsStr,
    fmt::Write,
    fs,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::{Path, PathBuf},
};
#[derive(Clone, Debug)]
pub struct Leaf {
    pub path: Vec<u8>,
    pub bytes: Vec<u8>,
    pub kind: Kind,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    File,
    Executable,
    Link,
}
#[derive(Debug)]
pub struct Golden {
    pub name: String,
    pub tree: Vec<Leaf>,
    root: PathBuf,
}
#[derive(Debug, Default)]
pub struct Verdict {
    differences: Vec<String>,
}
impl Verdict {
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.differences.is_empty()
    }
    #[must_use]
    pub fn report(&self) -> String {
        self.differences.join("\n")
    }
    #[must_use]
    pub fn changed(&self) -> String {
        self.report()
    }
}
#[must_use]
pub fn asked(value: Option<&OsStr>) -> bool {
    value == Some(OsStr::new("1"))
}
#[must_use]
pub fn bless() -> bool {
    asked(std::env::var_os("HEARTH_BLESS").as_deref())
}
#[must_use]
pub fn load(name: &str) -> Golden {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
    load_from(name, &root)
}
#[must_use]
pub fn load_from(name: &str, files: &Path) -> Golden {
    let mut tree = Vec::new();
    leaves(files, files, &mut tree);
    Golden { name: name.to_owned(), tree, root: Path::new(env!("CARGO_MANIFEST_DIR")).join(name) }
}
fn leaves(root: &Path, directory: &Path, out: &mut Vec<Leaf>) {
    let mut entries: Vec<_> =
        fs::read_dir(directory).expect("golden directory").map(|e| e.expect("golden entry")).collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let name = entry.file_name();
        if directory == root && (name.as_bytes().starts_with(b"expected") || name == "rendered" || name == ".git") {
            continue;
        }
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).expect("golden metadata");
        if metadata.is_dir() {
            leaves(root, &path, out);
            continue;
        }
        let (kind, bytes) = if metadata.file_type().is_symlink() {
            (Kind::Link, fs::read_link(&path).expect("golden link").as_os_str().as_bytes().to_vec())
        } else {
            (
                if metadata.permissions().mode() & 0o111 == 0 { Kind::File } else { Kind::Executable },
                fs::read(&path).expect("golden file"),
            )
        };
        out.push(Leaf {
            path: path.strip_prefix(root).expect("below golden").as_os_str().as_bytes().to_vec(),
            bytes,
            kind,
        });
    }
}
impl Golden {
    fn expected(&self, variant: &str) -> PathBuf {
        self.root.join(if variant.is_empty() { "expected".to_owned() } else { format!("expected-{variant}") })
    }
    #[must_use]
    pub fn targets(stdout: &str) -> Vec<(String, String)> {
        let mut targets: Vec<(String, String)> = Vec::new();
        for line in stdout.split_inclusive('\n') {
            if let Some(name) = line.strip_prefix("target ").and_then(|line| line.split_whitespace().next()) {
                targets.push((name.to_owned(), String::new()));
            }
            if let Some((_, text)) = targets.last_mut() {
                text.push_str(line);
            }
        }
        targets
    }
    fn outputs(&self, variant: &str, stdout: &str) -> Vec<(String, String)> {
        if self.expected(variant).join("output").exists() || self.name.starts_with("errors/") {
            vec![("output".to_owned(), stdout.to_owned())]
        } else {
            Self::targets(stdout)
        }
    }
    #[must_use]
    pub fn check(&self, variant: &str, stdout: &str, exit: Option<u8>) -> Verdict {
        let expected = self.expected(variant);
        let mut verdict = Verdict::default();
        let outputs = self.outputs(variant, stdout);
        if outputs.is_empty() {
            verdict.differences.push(format!("{}: no target output", self.name));
        }
        for (name, text) in &outputs {
            let path = expected.join(name);
            match fs::read_to_string(&path) {
                Ok(written) if written == *text => {}
                Ok(_) => verdict.differences.push(format!("{} differs\n{text}", path.display())),
                Err(_) => verdict.differences.push(format!("{} missing", path.display())),
            }
        }
        if expected.is_dir() {
            for entry in fs::read_dir(&expected).expect("expected directory") {
                let name = entry.expect("expected entry").file_name().into_string().expect("target name");
                if name != "exit" && !outputs.iter().any(|(target, _)| *target == name) {
                    verdict.differences.push(format!("{} has no output", expected.join(name).display()));
                }
            }
        }
        let code = fs::read_to_string(expected.join("exit")).map_or(Some(0), |s| s.trim().parse::<u8>().ok());
        if exit != code {
            verdict.differences.push(format!("{}: exit {exit:?}, expected {code:?}", self.name));
        }
        verdict.differences.extend(self.witnesses(stdout).differences);
        verdict
    }
    pub fn bless(&self, variant: &str, stdout: &str, exit: Option<u8>) {
        let expected = self.expected(variant);
        fs::create_dir_all(&expected).expect("expected directory");
        let outputs = self.outputs(variant, stdout);
        for entry in fs::read_dir(&expected).expect("expected files") {
            fs::remove_file(entry.expect("expected file").path()).expect("remove previous expectation");
        }
        for (name, text) in outputs {
            fs::write(expected.join(name), text).expect("blessed output");
        }
        if exit != Some(0) {
            fs::write(expected.join("exit"), format!("{}\n", exit.expect("ordinary exit"))).expect("blessed exit");
        }
    }
    #[must_use]
    pub fn witnesses(&self, stdout: &str) -> Verdict {
        let mut verdict = Verdict::default();
        let root = self.root.join("rendered");
        if !root.exists() {
            return verdict;
        }
        let mut witnesses = Vec::new();
        leaves(&root, &root, &mut witnesses);
        let targets = Self::targets(stdout);
        for leaf in witnesses {
            let name = std::str::from_utf8(&leaf.path).expect("witness path");
            let (target, path) = name.split_once('/').expect("witness target/path");
            let path = format!("/{path}");
            let text = targets.iter().find(|(name, _)| name == target).map_or("", |(_, text)| text.as_str());
            let mut digest = String::new();
            for byte in skein_lib::sha256::digest(&leaf.bytes).iter().take(6) {
                write!(digest, "{byte:02x}").expect("hex");
            }
            if !text.lines().any(|line| witness_line(line, &path, &digest)) {
                verdict.differences.push(format!("rendered/{name}: no file line ending {digest}"));
            }
        }
        verdict
    }
    /// Read all target names from the reviewed expected files, including cluster members.
    #[must_use]
    pub fn target_digests(&self, variant: &str) -> Vec<(String, [u8; 6])> {
        let mut targets: Vec<_> = fs::read_dir(self.expected(variant))
            .expect("reviewed expectations")
            .map(|entry| entry.expect("expected target").file_name().into_string().expect("target name"))
            .filter_map(|name| self.spec_digest(variant, &name).map(|digest| (name, digest)))
            .collect();
        targets.sort_by(|a, b| a.0.cmp(&b.0));
        targets
    }
    #[must_use]
    pub fn spec_digest(&self, variant: &str, target: &str) -> Option<[u8; 6]> {
        let text = fs::read_to_string(self.expected(variant).join(target)).ok()?;
        let hex = text.lines().find_map(|line| line.strip_prefix("spec "))?.trim();
        let bytes: Result<Vec<_>, _> = hex
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).expect("hex"), 16))
            .collect();
        bytes.ok()?.try_into().ok()
    }
}
#[must_use]
pub fn mask12(text: &str) -> String {
    let mut out = String::new();
    let mut hex = String::new();
    for character in text.chars().chain(core::iter::once('\0')) {
        if character.is_ascii_digit() || ('a'..='f').contains(&character) {
            hex.push(character);
            continue;
        }
        if hex.len() == 12 {
            out.push_str("............");
        } else {
            out.push_str(&hex);
        }
        hex.clear();
        if character != '\0' {
            out.push(character);
        }
    }
    out
}
mod repository;

/// A file witness names its path and exactly twelve digest digits.
#[must_use]
pub fn witness_line(line: &str, path: &str, digest: &str) -> bool {
    let mut words = line.split_whitespace();
    words.next() == Some(path) && words.last() == Some(digest) && digest.len() == 12
}
