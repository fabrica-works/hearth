//! Input stories at the command's retained-spec allowance.
use crate::{
    call::{Call, Side},
    limits::SHIPPED,
    vocabulary::{Answer, Output, Request},
};
use skein_lib::{List, Map, Queue, bytes};
// The spec allowance is the design's 16 MiB. Two half-budget strings pass it together.
pub const SHIPPED_SPEC_BYTES: u64 = 16_777_216;
pub const SPEC_BYTES: u64 = 16_384; // A KiB for each shipped MiB keeps hashing cheap.
const LIMITS: crate::limits::Limits = crate::limits::Limits { specs: SPEC_BYTES, ..SHIPPED };
pub const TOO_MANY_BYTES: u64 = SPEC_BYTES + 1;
pub const HALF_BYTES: u64 = SPEC_BYTES / 2 + 1;
const PREPARED_TARGETS: u32 = 1; // The cursor skips the one injected target.
const SPEC_STEPS: u32 = 128; // Two targets' additions, keep/find handshakes and final comparison.
pub struct Story {
    pub call: Call,
    pub failures: Box<[(Box<[u8]>, crate::diff::Side)]>,
}
fn prepare(side: &mut Side, param_bytes: u64) {
    let (base, mut target) = super::removed_task();
    target.base_env = base.documents.find(b"hosts/x1/x1-prod.json").expect("target path");
    side.documents = base.documents;
    side.entries = base.entries;
    side.targets = Box::new([target]);
    side.next = PREPARED_TARGETS;
    side.commit = Some(crate::revision::Oid(bytes::copy_of(b"commit")));
    side.files = true;
    side.ready = true;
    side.made = Map::with_capacity(SHIPPED.targets);
    let mut prepared = super::template_prepared(Box::new([]), crate::entries::Expansion { tasks: Box::new([]) });
    prepared.boot.params = Box::new([bytes::zeroed(usize::try_from(param_bytes).expect("spec size"))]);
    side.rendering = Some(crate::call::templates::Target::new(
        super::meta(),
        Box::new([]),
        super::KEY_A,
        crate::schema::Validated { tasks: Box::new([]) },
        super::Roots::x1_prod().debian,
        prepared,
        super::SINGLE_REQUEST,
    ));
}
pub fn render() -> Story {
    let mut call = Call::new(bytes::copy_of(b"/repo"), Box::new([]), &LIMITS);
    prepare(&mut call.side, TOO_MANY_BYTES);
    finish(call, TOO_MANY_BYTES)
}
pub fn diff() -> Story {
    let mut call = Call::diff(bytes::copy_of(b"/repo"), Box::new([]), bytes::copy_of(b"after"), &LIMITS);
    prepare(&mut call.side, HALF_BYTES);
    finish(call, HALF_BYTES)
}
fn finish(mut call: Call, param_bytes: u64) -> Story {
    let mut output = Queue::with_capacity(super::SINGLE_REQUEST);
    let mut failures = List::with_capacity(super::SINGLE_REQUEST);
    for _ in 0..SPEC_STEPS {
        if call.resume(LIMITS, &mut output) {
            return Story { call, failures: failures.into_boxed() };
        }
        match output.pop() {
            Some(Output::Request(Request::List { .. })) => prepare(&mut call.side, param_bytes),
            Some(Output::Request(Request::Keep { sha256, .. })) => call.side.kept(Some(sha256)),
            Some(Output::Request(Request::Find { .. })) => call.side.found(None, false),
            Some(Output::Answer(Answer::Change(change))) => {
                if let crate::diff::TargetChange::Failed { side } = change.change {
                    failures.push((change.target, side)).expect("one changed target");
                }
            }
            Some(Output::Request(_) | Output::Answer(_)) | None => {}
        }
    }
    panic!("spec fixture completes")
}
pub fn limit_error(side: Option<crate::diff::Side>) -> crate::errors::Error {
    crate::errors::Error {
        side,
        kind: crate::errors::Kind::Evaluation(crate::evaluate::FaultKind::Limit),
        class: crate::vocabulary::Class::Configuration,
        at: Some(crate::errors::Place { path: bytes::copy_of(b"hosts/x1/x1-prod.json"), position: None }),
        also: None,
        target: Some(bytes::copy_of(b"x1-prod")),
        task: None,
    }
}

pub fn finding(diff: bool) -> Side {
    let mut side = super::resolving_side();
    side.diff = diff;
    side.finding = crate::call::resolution::Finding::new(SHIPPED.targets);
    drop(super::next_resolution(&mut side));
    side
}
