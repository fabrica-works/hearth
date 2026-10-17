use alloc::boxed::Box;
pub fn finish_render(
    render: &mut crate::Render,
    env: &skein_lib::Env<crate::limits::Limits>,
    out: &mut skein_lib::Queue<crate::vocabulary::Output>,
) -> Box<[crate::vocabulary::Answer]> {
    let mut answers = skein_lib::List::with_capacity(env.limits.errors.saturating_add(10));
    for _ in 0..env.limits.errors.saturating_add(20) {
        if !render.work_pending() {
            break;
        }
        crate::resume(render, env, out);
        match out.pop() {
            Some(crate::vocabulary::Output::Answer(answer)) => {
                answers.push(answer).expect("answer capacity");
            }
            Some(crate::vocabulary::Output::Request(crate::vocabulary::Request::Find { request, .. })) => {
                crate::step(render, env, crate::vocabulary::Event::Absent { request }, out);
            }
            Some(crate::vocabulary::Output::Request(_)) | None => {}
        }
    }
    answers.into_boxed()
}

pub const KEY_A: hearth_lib::encode::Digest = hearth_lib::encode::Digest([1; 32]);
pub const KEY_B: hearth_lib::encode::Digest = hearth_lib::encode::Digest([2; 32]);
pub fn finding_side() -> crate::call::Side {
    use crate::{diff::Made, keys::Resolution};
    let mut side = crate::call::Side::new(b"/repo".as_slice().into(), Box::new([]), &crate::limits::SHIPPED);
    side.ready = true;
    side.files = true;
    side.made = skein_lib::Map::with_capacity(3);
    for (name, key) in [(b"c".as_slice(), KEY_A), (b"a", KEY_A), (b"b", KEY_B)] {
        let mut spec = super::spec(name, &[]);
        spec.resolution = Some(Box::new(Resolution::Needed { key }));
        side.made.insert(name.into(), Made::Spec(Box::new(spec))).expect("fixture targets");
    }
    side
}
pub fn key_inputs() -> crate::parts::packages::Inputs {
    use hearth_lib::encode::Item;
    crate::parts::packages::Inputs {
        sources: Box::new([Item::Map(0)]),
        request: Box::new([Item::List(1), Item::Bytes(b"hello".as_slice().into())]),
        variant: Box::new([Item::Bytes(b"apt".as_slice().into())]),
        apt: Box::new([Item::Map(0)]),
    }
}
fn packages_side(variant: crate::parts::packages::Variant) -> crate::call::Side {
    let mut side = finding_side();
    let mut made = skein_lib::Map::with_capacity(3);
    for (name, spec) in &side.made {
        let crate::diff::Made::Spec(spec) = spec else { panic!("fixture spec") };
        let mut spec = spec.as_ref().clone();
        let mut packages = super::thin_packages();
        packages.variant = variant;
        spec.parts =
            Box::new([crate::parts::Part { digest: KEY_A, record: crate::parts::PartRecord::Packages(packages) }]);
        made.insert(name.clone(), crate::diff::Made::Spec(Box::new(spec))).expect("fixture targets");
    }
    side.made = made;
    side.finding.resolve(Some(skein_lib::Time::ZERO.saturating_add(crate::limits::SHIPPED.resolve)));
    side
}
pub fn resolving_side() -> crate::call::Side {
    let mut side = packages_side(crate::parts::packages::Variant::Apt);
    let mut out = skein_lib::Queue::with_capacity(1);
    for _ in 0_u32..2 {
        drop(side.resolved_answer(&mut out));
        drop(out.pop());
        side.found(None, false);
    }
    side
}
pub fn container_resolving_side() -> crate::call::Side {
    packages_side(crate::parts::packages::Variant::Container)
}
pub fn next_resolution(side: &mut crate::call::Side) -> Option<crate::vocabulary::Request> {
    let mut out = skein_lib::Queue::with_capacity(1);
    drop(side.resolved_answer(&mut out));
    match out.pop() {
        Some(crate::vocabulary::Output::Request(request)) => Some(request),
        Some(crate::vocabulary::Output::Answer(_)) | None => None,
    }
}
#[derive(Clone, Copy, Debug)]
pub enum RunCause {
    Unknown,
    Unsatisfiable,
    Absent,
    Fetch,
    Resolve,
    Space,
    Start,
    Timeout,
    Other,
}
pub fn run_failure(cause: RunCause) -> (crate::vocabulary::Class, crate::vocabulary::Failure) {
    use crate::{
        errors::{Command, Ended, Tool},
        resolution::Cause,
        vocabulary::{Class, Failure},
    };
    let (ended, cause) = match cause {
        RunCause::Unknown => (Ended::Exit(1), Some(Cause::Unknown(b"apt".as_slice().into()))),
        RunCause::Unsatisfiable => (Ended::Exit(1), Some(Cause::Unsatisfiable(None))),
        RunCause::Absent => (
            Ended::Exit(1),
            Some(Cause::Absent { uri: b"http://one/".as_slice().into(), suite: b"forky".as_slice().into() }),
        ),
        RunCause::Fetch => (Ended::Exit(1), Some(Cause::Fetch)),
        RunCause::Resolve => (Ended::Exit(1), Some(Cause::Resolve)),
        RunCause::Space => (Ended::Exit(1), Some(Cause::Space)),
        RunCause::Other => (Ended::Exit(1), Some(Cause::Other)),
        RunCause::Start => (Ended::Start, None),
        RunCause::Timeout => (Ended::Timeout, None),
    };
    (
        match &cause {
            Some(cause) => cause.class(),
            None => Class::Operational,
        },
        Failure::Run { tool: Tool { command: Command::Mmdebstrap, ended, tail: Box::new([]) }, cause },
    )
}

pub const CLOSURE_LIMIT: u32 = 8_192;
pub const RESOLVE_TIME: skein_lib::Duration = skein_lib::Duration::from_secs(600);
pub const RESOLVE_START: skein_lib::Time = skein_lib::Time::from_nanos(13_000_000_000);
pub const RESOLVE_DEADLINE: skein_lib::Time = skein_lib::Time::from_nanos(613_000_000_000);
pub fn resolving_side_at(now: skein_lib::Time) -> crate::call::Side {
    use crate::vocabulary::Event;
    let env = skein_lib::Env { now, wall: skein_lib::Wall::EPOCH, limits: crate::limits::SHIPPED };
    let mut render = crate::Render::new(&env.limits);
    let mut out = skein_lib::Queue::with_capacity(1);
    crate::step(
        &mut render,
        &env,
        Event::Render {
            resolve: true,
            repository: b"/repo".as_slice().into(),
            revision: b"HEAD".as_slice().into(),
            targets: Box::new([]),
        },
        &mut out,
    );
    drop(out.pop());
    let mut side = render.call.take().expect("active call").side;
    side.made = packages_side(crate::parts::packages::Variant::Apt).made;
    side.ready = true;
    side.files = true;
    for _ in 0_u32..2 {
        drop(side.resolved_answer(&mut out));
        drop(out.pop());
        side.found(None, false);
    }
    side
}

pub const SMALL_CLOSURE: crate::limits::Limits = crate::limits::Limits { closure: 2, ..crate::limits::SHIPPED };
pub const APT_SHA256: hearth_lib::encode::Digest = hearth_lib::encode::Digest([
    0xc9, 0xc5, 0x69, 0x7b, 0x1b, 0xfa, 0x54, 0xd5, 0x70, 0x63, 0x72, 0x49, 0xb7, 0x8a, 0xee, 0xd3, 0x75, 0x51, 0x01,
    0x4f, 0xdf, 0xb6, 0x6d, 0xc4, 0x33, 0x48, 0x27, 0xf7, 0xc9, 0xb5, 0x47, 0x16,
]);
pub const APT_CLOSURE: hearth_lib::encode::Digest = hearth_lib::encode::Digest([
    0x90, 0x8a, 0x3b, 0x8a, 0x1d, 0xdd, 0x78, 0x10, 0x0f, 0x9f, 0x44, 0xb2, 0xd3, 0xe6, 0xb3, 0xf4, 0xed, 0xfd, 0x65,
    0xb2, 0x26, 0xdf, 0x34, 0x8e, 0x50, 0x21, 0xb5, 0xd6, 0xd3, 0x7f, 0x63, 0x8b,
]);
pub fn resolution_inst(name: &[u8], architecture: &[u8]) -> crate::resolution::Inst {
    crate::resolution::Inst {
        name: name.into(),
        architecture: architecture.into(),
        version: b"3.3.3".as_slice().into(),
    }
}
pub fn resolution_stanza(name: &[u8], architecture: &[u8]) -> crate::resolution::Stanza {
    crate::resolution::Stanza {
        package: name.into(),
        architecture: architecture.into(),
        version: b"3.3.3".as_slice().into(),
        filename: b"pool/main/a/apt/apt_3.3.3_amd64.deb".as_slice().into(),
        size: 1_567_912,
        sha256: APT_SHA256,
    }
}
pub fn apt_closure() -> crate::resolution::Closure {
    let mut closure = crate::resolution::Closure::new(&crate::limits::SHIPPED);
    closure.inst(resolution_inst(b"apt", b"amd64")).expect("installed apt");
    closure
}
pub fn matched_apt_closure() -> crate::resolution::Closure {
    let mut closure = apt_closure();
    closure.stanza(0, resolution_stanza(b"apt", b"amd64")).expect("matching apt");
    closure
}
