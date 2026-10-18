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
    let golden = golden::load(name);
    let files = files_of(&golden.tree);
    let repository = Repository::make(name, &files);
    super::lay(&repository.scratch().path().join("store"), laid);
    let out = super::render(&repository, &[], hearth::Form::Text);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let verdict = golden.check(variant, &stdout, exit_code(&out));
    if golden::bless() {
        golden.bless(variant, &stdout, exit_code(&out));
        panic!("blessed: {}", verdict.changed());
    }
    let joint = crate::protocol::run(&scenario_of(&files, laid, None), GOLDEN_SEED);
    let joint = golden.check(variant, &text(&joint), joint.code);
    assert!(joint.is_clean(), "joint: {}", joint.report());
    assert!(verdict.is_clean(), "{}", verdict.report());
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
    Repository::make(name, &files_of(&golden::load(name).tree))
}
/// A production-only hostname edit, leaving the shared carbon base intact.
pub const CARBON_PRODUCTION_EDIT: File = File {
    path: "hosts/carbon/carbon-prod.json",
    body: br#"{"placement":{"kind":"metal","firmware":"efi","hardware":"serverhino"},"core":{"hostname":"carbon-production"}}"#,
    mode: 0o644,
};
