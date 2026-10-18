//! Shared process and protocol checks for reviewed goldens.
use super::{File, Laid, Rendered, Repository, Scenario};
use hearth_render_world::golden::{self, Kind, Leaf};
#[must_use]
pub fn files_of(leaves: &[Leaf]) -> Vec<File> {
    leaves
        .iter()
        .map(|leaf| File {
            path: String::from_utf8(leaf.path.clone()).expect("golden path").leak(),
            body: leaf.bytes.clone().leak(),
            mode: match leaf.kind {
                Kind::File => 0o644,
                Kind::Executable => 0o755,
                Kind::Link => 0o120_000,
            },
        })
        .collect()
}
#[must_use]
pub fn exit_code(rendered: &Rendered) -> Option<u8> {
    match rendered.exit {
        skein_io::kernel::Exit::Code(code) => Some(code),
        skein_io::kernel::Exit::Signal(_) => None,
    }
}
#[must_use]
pub fn scenario_of<'a>(files: &'a [File], laid: &'a [Laid], fault: Option<super::Fault>) -> Scenario<'a> {
    Scenario { files, laid, fault, ..super::thin() }
}
#[must_use]
pub fn text(run: &crate::protocol::Run) -> String {
    String::from_utf8(run.stdout.clone()).expect("client text")
}
/// The same expectation and witnesses apply at both integration boundaries.
pub fn check_golden(name: &str, variant: &str, laid: &[Laid]) {
    check_loaded_golden(&golden::load(name), variant, laid);
}
/// Apply the same process and protocol checks to a repository loaded in place.
pub fn check_loaded_golden(golden: &golden::Golden, variant: &str, laid: &[Laid]) {
    let files = files_of(&golden.tree);
    let repository = repository_for(golden);
    super::lay(&repository.scratch().path().join("store"), laid);
    let out = super::render(&repository, &[], hearth::Form::Text);
    let stdout = String::from_utf8_lossy(&out.stdout);
    check_output(golden, variant, &stdout, exit_code(&out));
    let joint = crate::protocol::run(&scenario_of(&files, laid, None), GOLDEN_SEED);
    check_output(golden, variant, &text(&joint), joint.code);
}
/// The first deterministic protocol schedule; domain goldens sample four schedules.
pub const GOLDEN_SEED: u64 = 0;
#[must_use]
pub fn golden_output(name: &str) -> String {
    let repository = golden_repository(name);
    String::from_utf8(super::render(&repository, &[], hearth::Form::Text).stdout).expect("client text")
}
#[must_use]
pub fn golden_repository(name: &str) -> Repository {
    repository_for(&golden::load(name))
}
/// A production-only hostname edit, leaving the shared carbon base intact.
pub const CARBON_PRODUCTION_EDIT: File = File {
    path: "hosts/carbon/carbon-prod.json",
    body: br#"{"placement":{"kind":"metal","firmware":"efi","hardware":"serverhino"},"core":{"hostname":"carbon-production"}}"#,
    mode: 0o644,
};

/// Repository configuration failures, followed by the multiple-fault ordering case.
pub const ERROR_CASES: &[&str] = &[
    "errors/decode",
    "errors/repository",
    "errors/composition",
    "errors/schema",
    "errors/evaluation",
    "errors/guard",
    "errors/reserved",
    "errors/ids",
    "errors/collision",
    "errors/exported",
    "errors/sorted",
];
#[derive(Clone, Copy)]
pub enum GoldenCall {
    KilledGit,
    Resolve,
}
pub const PROTOCOL_ERRORS: &[(&str, GoldenCall, Option<u8>)] =
    &[("errors/tool", GoldenCall::KilledGit, Some(75)), ("errors/resolution", GoldenCall::Resolve, Some(2))];
#[must_use]
pub fn protocol_golden(name: &str, call: GoldenCall) -> ProtocolGolden {
    let golden = golden::load(name);
    let files = files_of(&golden.tree);
    let scenario = match call {
        GoldenCall::KilledGit => {
            scenario_of(&files, &[], Some(super::Fault::KilledAt { stdout_bytes: super::KILLED_BYTES }))
        }
        GoldenCall::Resolve => super::resolving(&files),
    };
    let run = crate::protocol::run(&scenario, GOLDEN_SEED);
    ProtocolGolden { golden, run }
}
pub struct ProtocolGolden {
    golden: golden::Golden,
    pub run: crate::protocol::Run,
}
impl ProtocolGolden {
    pub fn check(&self) {
        check_output(&self.golden, "", &text(&self.run), self.run.code);
    }
}

#[must_use]
fn repository_for(golden: &golden::Golden) -> Repository {
    Repository::make(&golden.name.replace('/', "-"), &files_of(&golden.tree))
}

fn check_output(golden: &golden::Golden, variant: &str, stdout: &str, code: Option<u8>) {
    let verdict = golden.check(variant, stdout, code);
    if golden::bless() {
        golden.bless(variant, stdout, code);
        panic!("blessed: {}", verdict.changed());
    }
    assert!(verdict.is_clean(), "{}: {}", golden.name, verdict.report());
}

/// render.md 7.2's repository-caused kinds plus the two builder-world kinds.
pub const ERROR_KINDS: &[&str] = &[
    "collision",
    "composition",
    "decode",
    "evaluation",
    "exported",
    "guard",
    "ids",
    "repository",
    "reserved",
    "resolution",
    "schema",
    "tool",
];

/// The pinned test repository supplies files; the golden holds expectations alone.
#[must_use]
pub fn resolution_golden() -> golden::Golden {
    let files = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testing/repos/test");
    golden::load_from("resolution", &files)
}

/// Whole digests from client JSON-lines spec records (client/README.md, 3.2).
#[must_use]
pub fn digests_of(text: &str) -> Vec<(String, String)> {
    let mut digests = Vec::new();
    for line in text.lines().filter(|line| line.contains("\"record\":\"spec\"")) {
        let field = |name| {
            let prefix = format!("\"{name}\":\"");
            line.split_once(&prefix).expect("spec field").1.split('"').next().expect("field value").to_owned()
        };
        digests.push((field("target"), field("digest")));
    }
    digests.sort();
    digests
}
