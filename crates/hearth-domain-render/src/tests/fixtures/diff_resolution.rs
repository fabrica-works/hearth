use crate::{
    diff::Made,
    resolution::Package,
    vocabulary::{Output, Request},
};
use hearth_lib::encode::Digest;
use skein_lib::{Queue, Token};
pub struct Closures {
    pub before: Box<[Package]>,
    pub after: Box<[Package]>,
}
pub fn closure_package(name: &[u8], architecture: &[u8], version: &[u8]) -> Package {
    let stanza = super::resolution_stanza(name, architecture);
    Package {
        name: stanza.package,
        architecture: stanza.architecture,
        version: version.into(),
        source: 0,
        filename: stanza.filename,
        size: stanza.size,
        sha256: stanza.sha256,
    }
}
pub fn changed_closures() -> Closures {
    Closures {
        before: Box::new([
            closure_package(b"apt", b"all", b"1"),
            closure_package(b"apt", b"amd64", b"2"),
            closure_package(b"nano", b"amd64", b"8"),
        ]),
        after: Box::new([
            closure_package(b"apt", b"all", b"1"),
            closure_package(b"apt", b"amd64", b"3"),
            closure_package(b"vim", b"amd64", b"9"),
        ]),
    }
}
pub fn diff_record(key: Digest, closure: Box<[Package]>) -> crate::resolution::Record {
    crate::resolution::Record {
        key,
        inputs: super::key_inputs(),
        sources: Box::new([]),
        closure,
        digest: key,
        made: crate::resolution::Made {
            at: skein_lib::Wall::EPOCH,
            tools: crate::resolution::Tools { apt: Box::new([]), mmdebstrap: Box::new([]) },
            costs: crate::resolution::Costs { fetched: 0, took: skein_lib::Duration::ZERO },
        },
    }
}
fn diff_side(key: Digest, recorded: bool, closure: Box<[Package]>) -> crate::call::Side {
    let mut side = crate::call::Side::new(b"/repo".as_slice().into(), Box::new([]), &crate::limits::SHIPPED);
    side.made = skein_lib::Map::with_capacity(1);
    side.diff = true;
    side.files = true;
    side.ready = true;
    side.commit = Some(crate::revision::Oid(b"commit".as_slice().into()));
    let spec = diff_spec(b"x1-prod", key);
    side.made.insert(b"x1-prod".as_slice().into(), Made::Spec(Box::new(spec))).expect("one target");
    assert_eq!(super::next_resolution(&mut side), Some(Request::Find { request: Token::new(5), key }));
    side.found(if recorded { Some(diff_record(key, closure)) } else { None }, false);
    side
}
pub fn resolution_diff(
    before: bool,
    after: bool,
    resolve: bool,
) -> (crate::Render, skein_lib::Env<crate::limits::Limits>, Queue<Output>) {
    let Closures { before: old, after: new } = changed_closures();
    diff_from_sides(diff_side(super::KEY_A, before, old), diff_side(super::KEY_B, after, new), resolve)
}
fn diff_from_sides(
    before: crate::call::Side,
    after: crate::call::Side,
    resolve: bool,
) -> (crate::Render, skein_lib::Env<crate::limits::Limits>, Queue<Output>) {
    let (mut render, env, mut out) = super::diff_call();
    drop(out.pop());
    render.call.as_mut().expect("call").side = before;
    let request = next_diff_output(&mut render, &env, &mut out);
    assert_eq!(
        request,
        Output::Request(Request::List {
            request: Token::new(1),
            repository: b"/repo".as_slice().into(),
            revision: b"after".as_slice().into()
        })
    );
    let call = render.call.as_mut().expect("call");
    call.side = after;
    call.side.errors.side(crate::diff::Side::To);
    call.diff.as_mut().expect("diff").resolve(if resolve { Some(super::RESOLVE_DEADLINE) } else { None });
    (render, env, out)
}
pub fn next_diff_output(
    render: &mut crate::Render,
    env: &skein_lib::Env<crate::limits::Limits>,
    out: &mut Queue<Output>,
) -> Output {
    for _ in 0_u32..32 {
        if let Some(output) = out.pop() {
            return output;
        }
        crate::resume(render, env, out);
    }
    panic!("a bounded diff advances")
}
pub fn diff_resolution(
    render: &mut crate::Render,
    env: &skein_lib::Env<crate::limits::Limits>,
    out: &mut Queue<Output>,
) -> crate::diff::ResolutionChange {
    let Output::Answer(crate::vocabulary::Answer::Change(change)) = next_diff_output(render, env, out) else {
        panic!("change")
    };
    let crate::diff::TargetChange::Changed { parts, .. } = change.change else { panic!("changed") };
    let crate::diff::PartChange::Packages(packages) = super::only(&parts) else { panic!("packages") };
    packages.resolution.clone()
}
pub fn complete_diff_resolution(
    render: &mut crate::Render,
    env: &skein_lib::Env<crate::limits::Limits>,
    out: &mut Queue<Output>,
    names: &[Box<[u8]>],
) {
    use crate::vocabulary::Event;
    for name in names {
        crate::step(
            render,
            env,
            Event::Inst { request: Token::new(6), inst: super::resolution_inst(name, b"amd64") },
            out,
        );
        crate::step(
            render,
            env,
            Event::Stanza { request: Token::new(6), line: 0, stanza: super::resolution_stanza(name, b"amd64") },
            out,
        );
    }
    let side = &mut render.call.as_mut().expect("call").side;
    let request = super::record_side(side).expect("record request");
    out.push(Output::Request(request));
}

fn diff_spec(name: &[u8], key: Digest) -> crate::parts::Spec {
    let mut spec = super::spec(name, &[]);
    spec.digest = key;
    let mut packages = super::thin_packages();
    packages.request =
        Box::new([if key == super::KEY_A { b"nano".as_slice().into() } else { b"vim".as_slice().into() }]);
    spec.parts = Box::new([
        super::software::empty_software_part(),
        crate::parts::Part { digest: key, record: crate::parts::PartRecord::Packages(packages) },
        crate::parts::Part { digest: super::KEY_A, record: crate::parts::PartRecord::Config(super::config()) },
    ]);
    spec.resolution = Some(Box::new(crate::keys::Resolution::Needed { key }));
    spec
}

fn crossed_side(first: Digest, second: Digest) -> crate::call::Side {
    let mut side = diff_side(first, false, Box::new([]));
    let mut made = skein_lib::Map::with_capacity(2);
    made.insert(b"x1-prod".as_slice().into(), Made::Spec(Box::new(diff_spec(b"x1-prod", first))))
        .expect("first target");
    made.insert(b"z1-prod".as_slice().into(), Made::Spec(Box::new(diff_spec(b"z1-prod", second))))
        .expect("second target");
    side.made = made;
    drop(super::next_resolution(&mut side));
    side.found(None, false);
    side
}
pub fn crossed_resolution_diff() -> (crate::Render, skein_lib::Env<crate::limits::Limits>, Queue<Output>) {
    diff_from_sides(crossed_side(super::KEY_A, super::KEY_B), crossed_side(super::KEY_B, super::KEY_A), true)
}
pub fn fail_diff_resolution(
    render: &mut crate::Render,
    env: &skein_lib::Env<crate::limits::Limits>,
    out: &mut Queue<Output>,
    failure: (crate::vocabulary::Class, crate::vocabulary::Failure),
) {
    let (class, failure) = failure;
    crate::step(
        render,
        env,
        crate::vocabulary::Event::Failed { request: Token::new(6), class, at: None, failure },
        out,
    );
}
