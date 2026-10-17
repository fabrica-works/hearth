//! The real resolve scenario through the pinned loopback mirror (testing.md, 5.2).
use crate::run::{self, Run};
use hearth_protocol_mmdebstrap::{
    limits::{self, Limits},
    vector::{self, Apt, Component, Name, Simulate, Sources, Timestamp, Variant},
};
use hearth_recordings::{Mask, Recording};
use hearth_test_mirror::{Mirror, Pin, Proxy, Served};
use skein_scratch::Scratch;
use std::{collections::BTreeSet, fs, net::TcpListener, path::PathBuf, process::Command};

/// One pinned target with optional request, timestamp and reachability changes.
#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub name: &'static str,
    pub target: &'static str,
    pub request: Option<&'static [&'static str]>,
    pub archive: Option<&'static str>,
    pub reach: Reach,
}
/// The harness proxy, or a loopback port with no listener.
#[derive(Clone, Copy, Debug)]
pub enum Reach {
    Proxy,
    Unreachable,
}
pub const CASES: &[Case] = &[
    Case { name: "test-min", target: "test-min", request: None, archive: None, reach: Reach::Proxy },
    Case { name: "test-boot", target: "test-boot", request: None, archive: None, reach: Reach::Proxy },
    Case {
        name: "unknown-package",
        target: "test-min",
        request: Some(&["hearth-absent"]),
        archive: None,
        reach: Reach::Proxy,
    },
    Case {
        name: "no-candidate",
        target: "test-min",
        request: Some(&["mail-transport-agent"]),
        archive: None,
        reach: Reach::Proxy,
    },
    Case {
        name: "unsatisfiable",
        target: "test-min",
        request: Some(&["exim4-daemon-light", "postfix"]),
        archive: None,
        reach: Reach::Proxy,
    },
    Case { name: "unreachable-mirror", target: "test-min", request: None, archive: None, reach: Reach::Unreachable },
    Case {
        name: "absent-snapshot",
        target: "test-min",
        request: None,
        archive: Some("20000101T000000Z"),
        reach: Reach::Proxy,
    },
];
#[must_use]
pub fn case(name: &str) -> &'static Case {
    CASES.iter().find(|case| case.name == name).expect("named resolve case")
}
/// Prerequisites and tool versions for resolve's real side.
pub struct Resolve {
    pin: Pin,
    mirror: Mirror,
    version: String,
}
impl Default for Resolve {
    fn default() -> Self {
        Self::new()
    }
}
impl Resolve {
    #[must_use]
    pub fn new() -> Self {
        let mmdebstrap = version("/usr/bin/mmdebstrap", "mmdebstrap 1.", "mmdebstrap ");
        let apt = version("/usr/bin/apt-get", "apt ", "apt ");
        let pin = Pin::committed();
        let mirror = Mirror::find(&pin).unwrap_or_else(|error| panic!("conformance requires the mirror: {error}"));
        assert!(
            mirror.index("main").is_file(),
            "conformance requires testing/mirror/fetch: checked main index missing"
        );
        Self { pin, mirror, version: format!("{mmdebstrap}-apt{apt}") }
    }
    #[must_use]
    pub fn simulate(&self, case: &Case) -> Simulate {
        inputs(&self.pin, case)
    }
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }
    #[must_use]
    pub fn run(&self, case: &Case) -> Resolved {
        self.run_under(case, &limits::SHIPPED)
    }
    #[must_use]
    pub fn run_under(&self, case: &Case, limits: &Limits) -> Resolved {
        let scratch = Scratch::new(case.name);
        let proxy = Proxy::serve(&self.mirror).expect("loopback mirror proxy");
        let port = match case.reach {
            Reach::Proxy => proxy.port(),
            Reach::Unreachable => TcpListener::bind("127.0.0.1:0")
                .expect("unused loopback port")
                .local_addr()
                .expect("loopback address")
                .port(),
        };
        let url = format!("http://127.0.0.1:{port}/");
        let proxy_value = vector::Proxy::new(url.as_bytes(), limits).expect("loopback proxy URI");
        let invocation = vector::simulate(
            &self.simulate(case),
            scratch.path().as_os_str().as_encoded_bytes(),
            Some(&proxy_value),
            limits,
        )
        .expect("pinned simulation vector");
        let mut run = run::mmdebstrap(&invocation, scratch.path());
        run.captured.version = self.version.clone().into();
        Resolved { run, served: proxy.served(), scratch, port }
    }
    #[must_use]
    pub fn index(&self) -> Vec<u8> {
        fs::read(self.mirror.index("main")).expect("checked main index")
    }
    pub fn settle(&self, case: &Case, recording: &Recording) {
        if hearth_recordings::recording_on() {
            hearth_recordings::write("mmdebstrap", &self.version, case.name, recording);
        } else {
            self.compare(case, recording);
        }
    }
    pub fn compare(&self, case: &Case, recording: &Recording) {
        if hearth_recordings::recording_on() {
            return;
        }
        let expected = hearth_recordings::read("mmdebstrap", &self.version, case.name);
        difference(case, "exit", recording.exit.as_bytes(), expected.exit.as_bytes());
        for (pipe, actual, expected) in
            [("stderr", &recording.stderr, &expected.stderr), ("stdout", &recording.stdout, &expected.stdout)]
        {
            difference(case, pipe, &significant(actual), &significant(expected));
        }
        for (name, actual, expected) in
            [("lists", &recording.lists, &expected.lists), ("stanzas", &recording.stanzas, &expected.stanzas)]
        {
            assert_eq!(actual.is_some(), expected.is_some(), "{} {name}: capture with HEARTH_RECORD=1", case.name);
            difference(case, name, actual.as_deref().unwrap_or_default(), expected.as_deref().unwrap_or_default());
        }
    }
}
/// A case's pinned inputs, shared by the real and fake vectors.
#[must_use]
pub fn inputs(pin: &Pin, case: &Case) -> Simulate {
    let target = pin.target(case.target).expect("pinned target");
    let name = |text: &str| Name::new(text.as_bytes(), &limits::SHIPPED).expect("pinned name");
    let request = case.request.map_or_else(
        || target.request.iter().map(|text| name(text)).collect(),
        |request| request.iter().map(|text| name(text)).collect(),
    );
    assert_eq!(&*target.variant, "apt", "M1's pinned variant");
    Simulate {
        sources: Sources {
            suite: name(&pin.suite),
            architecture: name(&pin.architecture),
            components: Box::new([Component::Main]),
            archive: Timestamp::new(case.archive.unwrap_or(&pin.archive).as_bytes()).expect("pinned timestamp"),
            updates: false,
            security: None,
        },
        request,
        variant: Variant::Apt,
        apt: Apt { install_recommends: false },
    }
}

fn version(tool: &str, prefix: &str, strip: &str) -> String {
    let output = Command::new(tool)
        .arg("--version")
        .output()
        .unwrap_or_else(|error| panic!("conformance requires {tool}: {error}"));
    assert!(output.status.success(), "conformance requires {tool} --version");
    let text = String::from_utf8(output.stdout).expect("tool version text");
    assert!(text.starts_with(prefix), "conformance requires {tool} printing {prefix}, found {text}");
    text.strip_prefix(strip).expect("version prefix").split_whitespace().next().expect("version number").into()
}
fn significant(pipe: &[u8]) -> Vec<u8> {
    pipe.split(|byte| *byte == b'\n')
        .filter(|line| [b"Inst ".as_slice(), b"E: ", b"Err:", b"W: "].iter().any(|prefix| line.starts_with(prefix)))
        .flat_map(|line| line.iter().copied().chain([b'\n']))
        .collect()
}
fn difference(case: &Case, field: &str, actual: &[u8], expected: &[u8]) {
    if actual != expected {
        let at = actual.iter().zip(expected).position(|(a, b)| a != b).unwrap_or(actual.len().min(expected.len()));
        panic!(
            "{} {field} differs at byte {at} (lengths {} and {}); capture with HEARTH_RECORD=1",
            case.name,
            actual.len(),
            expected.len()
        );
    }
}
/// One real run, its proxy answers and the scratch holding apt's lists.
pub struct Resolved {
    pub run: Run,
    pub served: Vec<Served>,
    scratch: Scratch,
    port: u16,
}
impl Resolved {
    #[must_use]
    pub fn inst_lines(&self) -> Vec<&[u8]> {
        self.run
            .captured
            .stderr
            .split(|byte| *byte == b'\n')
            .chain(self.run.captured.stdout.split(|byte| *byte == b'\n'))
            .filter(|line| line.starts_with(b"Inst "))
            .collect()
    }
    fn lists(&self) -> PathBuf {
        self.scratch.path().join("root/var/lib/apt/lists")
    }
    #[must_use]
    pub fn list(&self, component: &str) -> Option<Vec<u8>> {
        let suffix = format!("_{component}_binary-");
        let entries = fs::read_dir(self.lists()).ok()?;
        for entry in entries {
            let entry = entry.expect("apt list entry");
            let name = entry.file_name();
            let name = name.to_str().expect("apt list name");
            if name.contains(&suffix) && name.ends_with("_Packages") {
                return Some(fs::read(entry.path()).expect("apt's Packages list"));
            }
        }
        None
    }
    #[must_use]
    pub fn recording(&self) -> Recording {
        let mut recording = self.run.recording();
        let mut listing = Vec::new();
        match fs::read_dir(self.lists()) {
            Ok(entries) => {
                for entry in entries {
                    let entry = entry.expect("apt lists entry");
                    let metadata = entry.metadata().expect("apt list metadata");
                    let name = entry.file_name();
                    let name = name.to_str().expect("apt list name");
                    listing.push(if metadata.is_dir() {
                        format!("directory {name}\n")
                    } else {
                        format!("file {} {name}\n", metadata.len())
                    });
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => panic!("apt lists: {error}"),
        }
        listing.sort_by(|a, b| a.split_whitespace().last().cmp(&b.split_whitespace().last()));
        recording.lists = Some(listing.concat().into_bytes());
        Mask::new(self.scratch.path())
            .also(format!("127.0.0.1:{}", self.port).as_bytes(), b"127.0.0.1:PORT")
            .also(format!("127.0.0.1 {}", self.port).as_bytes(), b"127.0.0.1 PORT")
            .recording(&mut recording);
        recording
    }
}
/// Cut matching Package, Version and Architecture stanzas in index order.
///
/// # Errors
/// The first `Inst` line with no corresponding stanza, or a malformed line.
pub fn stanzas(list: &[u8], insts: &[&[u8]]) -> Result<Vec<u8>, Box<[u8]>> {
    let mut triples = Vec::new();
    for line in insts {
        triples.push((*line, triple(line)));
    }
    let requested: BTreeSet<_> = triples.iter().filter_map(|(_, triple)| *triple).collect();
    let mut found = BTreeSet::new();
    let mut result = Vec::new();
    let mut at = 0;
    while at < list.len() {
        let length = list[at..].windows(2).position(|pair| pair == b"\n\n").map_or(list.len() - at, |end| end + 2);
        let stanza = &list[at..at + length];
        let key = (field(stanza, b"Package: "), field(stanza, b"Version: "), field(stanza, b"Architecture: "));
        if requested.contains(&key) {
            result.extend_from_slice(stanza);
            found.insert(key);
        }
        at += length;
    }
    for (line, triple) in triples {
        if triple.is_none_or(|triple| !found.contains(&triple)) {
            return Err(line.into());
        }
    }
    Ok(result)
}
fn field<'a>(stanza: &'a [u8], key: &[u8]) -> &'a [u8] {
    stanza.split(|byte| *byte == b'\n').find_map(|line| line.strip_prefix(key)).unwrap_or_default()
}
fn triple(line: &[u8]) -> Option<(&[u8], &[u8], &[u8])> {
    let rest = line.strip_prefix(b"Inst ")?;
    let name = rest.split(|byte| *byte == b' ').next()?;
    let open = rest.iter().position(|byte| *byte == b'(')?;
    let rest = &rest[open + 1..];
    let version = rest.split(|byte| *byte == b' ').next()?;
    let end = rest.windows(2).position(|bytes| bytes == b"])")?;
    let start = rest[..end].iter().rposition(|byte| *byte == b'[')?;
    Some((name, version, &rest[start + 1..end]))
}
