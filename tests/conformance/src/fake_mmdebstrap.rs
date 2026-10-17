//! One resolve scenario on the fake, held to mmdebstrap's captured observations.
use crate::{
    fixtures::{OWNER, UNIVERSE, take},
    resolve::{self, Case, Reach},
};
use hearth_fake_machine::{
    Machine,
    hearth::{
        Tool,
        mirror::{Mirror as Served, Release, Suite, Timestamp},
        universe::{KEPT, Universe},
    },
    linux::{At, Exit, Handle, How, Kind, Op, Pid, Pipe, Reply, Sizes, Spawn, Way},
};
use hearth_fake_mmdebstrap::{
    process::{Faults, Mmdebstrap, Progress},
    resolve::{Request, Variant, read_by},
};
use hearth_protocol_mmdebstrap::{
    limits,
    tail::{self, Tail},
    vector,
};
use hearth_recordings::{Mask, Recording};
use hearth_test_mirror::{Mirror as Fetched, Pin};
use std::{collections::BTreeSet, fmt::Write, fs, path::Path};

pub const CONFORMANCE: Sizes = Sizes { files: 64, bytes: 256 << 20, processes: 8, pipe: 65_536, ledger: 4_096 };
pub const PORT: u16 = 3142;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum What {
    ListSizes,
    StanzaFields,
    ErrorLines,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Allowed {
    pub case: Option<&'static str>,
    pub what: What,
    pub reason: &'static str,
}
pub const ALLOWED: &[Allowed] = &[
    Allowed {
        case: None,
        what: What::ListSizes,
        reason: "The fake emits its own unsigned InRelease and KEPT index (testing.md, 3.6).",
    },
    Allowed {
        case: None,
        what: What::StanzaFields,
        reason: "The fake keeps KEPT fields; hearth reads its six fields (render.md, 5.3).",
    },
    Allowed {
        case: Some("unsatisfiable"),
        what: What::ErrorLines,
        reason: "apt's solver 3.0 explanation reports its internal conflicting decision tree, absent from the fake's marking model; the unmet-dependencies block is compared in full.",
    },
];
fn field<'a>(text: &'a [u8], name: &str) -> Option<&'a [u8]> {
    let key = format!("{name}: ");
    text.split(|b| *b == b'\n').find_map(|line| line.strip_prefix(key.as_bytes()))
}
fn release(text: &[u8]) -> Release {
    let required = |name| field(text, name).expect("pinned release field").into();
    Release {
        origin: required("Origin"),
        label: required("Label"),
        suite: required("Suite"),
        codename: required("Codename"),
        version: field(text, "Version").map(Into::into),
    }
}
fn served(pin: &Pin, release: Release, universe: Universe) -> Served {
    let mut mirror = Served::new();
    mirror.add(
        Timestamp::new(pin.archive.as_bytes()).expect("pinned timestamp"),
        Suite::new(release, pin.architecture.as_bytes(), Box::new([(b"main".as_slice().into(), universe)])),
    );
    mirror
}
fn fetched_release(mirror: &Fetched, pin: &Pin) -> Vec<u8> {
    let path = format!("archive/debian/{}/dists/{}/InRelease", pin.archive, pin.suite);
    let release = fs::read(mirror.file(Path::new(&path)).expect("pinned InRelease")).expect("release readable");
    let mut digest = String::new();
    for byte in skein_lib::sha256::digest(&release) {
        write!(&mut digest, "{byte:02x}").expect("digest string");
    }
    assert_eq!(digest, pin.digest.as_ref(), "pinned InRelease SHA-256");
    release
}
#[must_use]
pub fn mirror(index: &[u8]) -> Served {
    let pin = Pin::committed();
    let fetched = Fetched::find(&pin).expect("fetched mirror");
    served(&pin, release(&fetched_release(&fetched, &pin)), Universe::read(index).expect("pinned index"))
}
#[must_use]
pub fn committed() -> Served {
    let cut = fs::read(UNIVERSE).expect("committed universe; write under HEARTH_RECORD=1");
    let end = cut.windows(2).position(|p| p == b"\n\n").expect("release stanza");
    served(&Pin::committed(), release(&cut[..end]), Universe::read(&cut[end + 2..]).expect("committed KEPT index"))
}
#[must_use]
pub fn cut(mirror: &Fetched, pin: &Pin) -> Box<[u8]> {
    let universe = Universe::read(&fs::read(mirror.index("main")).expect("checked index")).expect("pinned universe");
    let mut read = BTreeSet::new();
    for case in resolve::CASES {
        let inputs = resolve::inputs(pin, case);
        let include: Box<[_]> = inputs.request.iter().map(|n| n.as_bytes().into()).collect();
        for variant in [Variant::Essential, Variant::Apt] {
            for selected in read_by(&Request { universes: &[&universe], variant, include: &include }) {
                read.insert(usize::try_from(selected.package).expect("index fits"));
            }
        }
    }
    let release = release(&fetched_release(mirror, pin));
    let mut cut = Vec::new();
    for (name, value) in [
        ("Origin", &release.origin),
        ("Label", &release.label),
        ("Suite", &release.suite),
        ("Codename", &release.codename),
    ] {
        cut.extend_from_slice(format!("{name}: ").as_bytes());
        cut.extend_from_slice(value);
        cut.push(b'\n');
    }
    if let Some(value) = release.version {
        cut.extend_from_slice(b"Version: ");
        cut.extend_from_slice(&value);
        cut.push(b'\n');
    }
    cut.push(b'\n');
    let packages = read.into_iter().map(|i| universe.packages()[i].clone()).collect();
    cut.extend_from_slice(&Universe::new(packages).index());
    cut.into()
}
/// The whole argv includes PROGRAM, as every captured vector does.
pub struct Spawned<'a> {
    pub argv: &'a [Vec<u8>],
    pub env: &'a [Vec<u8>],
    pub dir: &'a [u8],
}
#[must_use]
pub fn invocation(case: &Case) -> vector::Invocation {
    let port = match case.reach {
        Reach::Proxy => PORT,
        Reach::Unreachable => 9,
    };
    let url = format!("http://127.0.0.1:{port}/");
    let proxy = vector::Proxy::new(url.as_bytes(), &limits::SHIPPED).expect("proxy");
    vector::simulate(&resolve::inputs(&Pin::committed(), case), b"/scratch", Some(&proxy), &limits::SHIPPED)
        .expect("case vector")
}
fn open(machine: &mut Machine, parent: Pid, path: &[u8]) -> Handle {
    let Reply::Opened(handle) = machine
        .call(parent, Op::Open { at: At::Root, path: path.into(), how: How::Directory })
        .expect("open directory")
    else {
        panic!("directory handle");
    };
    handle
}
#[must_use]
pub fn run(mirror: &mut Option<Served>, case: &Case, spawned: &Spawned<'_>) -> Recording {
    let mut machine = Machine::new(46, CONFORMANCE);
    let source = invocation(case).sources_list;
    machine.linux_mut().lay(&[
        hearth_fake_machine::linux::Item::directory(b"/scratch", 0o755).owner(OWNER),
        hearth_fake_machine::linux::Item::file(b"/scratch/sources.list", 0o644, &source).owner(OWNER),
        hearth_fake_machine::linux::Item::directory(b"/usr", 0o755),
        hearth_fake_machine::linux::Item::directory(b"/usr/bin", 0o755),
    ]);
    machine.install(b"/usr/bin/mmdebstrap", Tool::Mmdebstrap);
    machine.serve(mirror.take().expect("mirror between runs"), Some(PORT));
    let parent = machine.linux_mut().login(OWNER, b"/scratch");
    let root = open(&mut machine, parent, b"/");
    let port = match case.reach {
        Reach::Proxy => PORT,
        Reach::Unreachable => 9,
    };
    let args = spawned.argv[1..]
        .iter()
        .map(|a| {
            String::from_utf8(a.clone()).expect("argv text").replace("PORT", &port.to_string()).into_bytes().into()
        })
        .collect();
    let Reply::Spawned { child, pipes } = machine
        .call(
            parent,
            Op::Spawn(Spawn {
                program: spawned.argv[0].as_slice().into(),
                args,
                env: spawned.env.iter().map(|e| e.clone().into()).collect(),
                root,
                dir: spawned.dir.strip_prefix(b"/").expect("absolute recorded directory").into(),
                pipes: Box::new([Pipe { child: 1, way: Way::Out }, Pipe { child: 2, way: Way::Out }]),
            }),
        )
        .expect("spawn fake mmdebstrap")
    else {
        panic!("spawned child");
    };
    let mut captured = Recording {
        version: String::new().into(),
        argv: spawned.argv.to_vec(),
        env: spawned.env.to_vec(),
        dir: spawned.dir.to_vec(),
        stdin: None,
        stdout: Vec::new(),
        stderr: Vec::new(),
        exit: String::new().into(),
        expect: None,
        lists: None,
        stanzas: None,
    };
    let mut child_process = Mmdebstrap::start(&mut machine, child, Faults { stall_after: None });
    let mut done = false;
    for _ in 0..10_000 {
        let progress = child_process.step(&mut machine);
        take(&mut machine, parent, pipes[0], &mut captured.stdout);
        take(&mut machine, parent, pipes[1], &mut captured.stderr);
        if progress == Progress::Exited {
            done = true;
            break;
        }
    }
    assert!(done, "bounded fake process settles");
    let Reply::Exited(exit) = machine.call(parent, Op::Wait { child }).expect("wait") else {
        panic!("exited child");
    };
    captured.exit = match exit {
        Exit::Code(code) => code.to_string().into(),
        Exit::Signal(signal) => format!("signal {signal}").into(),
    };
    artifacts(&mut machine, parent, &mut captured);
    *mirror = machine.unserve();
    Mask::new(Path::new("/scratch"))
        .also(format!("127.0.0.1:{port}").as_bytes(), b"127.0.0.1:PORT")
        .also(format!("127.0.0.1 {port}").as_bytes(), b"127.0.0.1 PORT")
        .recording(&mut captured);
    captured
}
fn artifacts(machine: &mut Machine, parent: Pid, captured: &mut Recording) {
    let path = b"/scratch/root/var/lib/apt/lists";
    let dir = open(machine, parent, path);
    let Reply::Listed(entries) = machine.call(parent, Op::List { dir, most: 64 }).expect("list apt indices") else {
        panic!("list entries");
    };
    let mut lists = Vec::new();
    for (kind, name) in entries {
        let full = [path.as_slice(), b"/", &name].concat();
        let name = String::from_utf8(name.into_vec()).expect("apt list name");
        match kind {
            Kind::Directory => lists.push(format!("directory {name}\n")),
            Kind::File => {
                let bytes = machine.linux().contents(&full).expect("list bytes");
                lists.push(format!("file {} {name}\n", bytes.len()));
                if name.ends_with("_Packages") {
                    captured.stanzas = Some(
                        resolve::stanzas(bytes, &insts(captured).iter().map(Vec::as_slice).collect::<Vec<_>>())
                            .expect("fake Inst stanzas"),
                    );
                }
            }
            Kind::Link => panic!("apt lists have no links"),
        }
    }
    lists.sort_by(|a, b| a.split_whitespace().last().cmp(&b.split_whitespace().last()));
    captured.lists = Some(lists.concat().into_bytes());
}

fn lines(recording: &Recording) -> impl Iterator<Item = &[u8]> {
    recording.stdout.split(|b| *b == b'\n').chain(recording.stderr.split(|b| *b == b'\n'))
}
#[must_use]
pub fn insts(recording: &Recording) -> Vec<Vec<u8>> {
    let mut lines: Vec<_> = lines(recording)
        .filter(|l| l.starts_with(b"Inst "))
        .map(|l| l[..l.windows(2).position(|p| p == b"])").expect("Inst ending") + 2].to_vec())
        .collect();
    lines.sort();
    lines.dedup();
    lines
}
fn without_solver<'a>(case: &Case, pipe: &'a [u8]) -> Vec<&'a [u8]> {
    let mut explanation = false;
    pipe.split(|b| *b == b'\n')
        .filter(|line| {
            if ALLOWED.iter().any(|row| row.case == Some(case.name) && row.what == What::ErrorLines)
                && line.starts_with(b"E: The following information from --solver 3.0")
            {
                explanation = true;
                return false;
            }
            if explanation && line.starts_with(b" ") {
                return false;
            }
            explanation = false;
            true
        })
        .collect()
}
fn failure(case: &Case, recording: &Recording) -> (tail::Cause, tail::Class, Vec<Vec<u8>>) {
    let tails: Vec<_> = [&recording.stdout, &recording.stderr]
        .iter()
        .map(|pipe| {
            let mut tail = Tail::new(&limits::SHIPPED);
            for line in without_solver(case, pipe) {
                tail::admit(&mut tail, line, &limits::SHIPPED);
            }
            tail
        })
        .collect();
    let tail = if tails[1].lines().is_empty() { &tails[0] } else { &tails[1] };
    (tail.cause().clone(), tail.cause().class(), tail.lines().into_iter().map(|l| l.to_vec()).collect())
}
fn kept(stanzas: &[u8]) -> Vec<Vec<Vec<u8>>> {
    let mut result = Vec::new();
    let mut fields = Vec::new();
    for line in stanzas.split(|b| *b == b'\n').chain(std::iter::once(b"".as_slice())) {
        if line.is_empty() {
            if !fields.is_empty() {
                fields.sort();
                result.push(std::mem::take(&mut fields));
            }
        } else if KEPT.iter().any(|key| line.starts_with(format!("{key}: ").as_bytes())) {
            fields.push(line.to_vec());
        }
    }
    result.sort();
    result
}
fn list_kinds(recording: &Recording) -> Vec<Vec<u8>> {
    recording
        .lists
        .as_deref()
        .expect("recorded lists")
        .split(|b| *b == b'\n')
        .filter(|l| !l.is_empty())
        .map(|line| {
            let fields: Vec<_> = line.split(|b| *b == b' ').collect();
            [fields[0], b" ", fields.last().expect("entry name")].concat()
        })
        .collect()
}
/// The first observation that differs beyond ALLOWED.
#[must_use]
pub fn differ(case: &Case, real: &Recording, fake: &Recording, stanzas: Option<&[u8]>) -> Option<String> {
    if real.exit != fake.exit {
        return Some(format!("exit: real {:?}, fake {:?}", real.exit, fake.exit));
    }
    if insts(real) != insts(fake) {
        return Some(format!(
            "Inst lines: real {:?}, fake {:?}",
            insts(real).iter().map(|l| String::from_utf8_lossy(l)).collect::<Vec<_>>(),
            insts(fake).iter().map(|l| String::from_utf8_lossy(l)).collect::<Vec<_>>()
        ));
    }
    for (name, real, fake) in [("stdout", &real.stdout, &fake.stdout), ("stderr", &real.stderr, &fake.stderr)] {
        let errors = |pipe: &[u8]| {
            without_solver(case, pipe)
                .into_iter()
                .filter(|line| {
                    [b"E: ".as_slice(), b"Err:", b"W: ", b"Error: ", b"Warning: "]
                        .iter()
                        .any(|prefix| line.starts_with(prefix))
                })
                .map(<[u8]>::to_vec)
                .collect::<Vec<_>>()
        };
        if errors(real) != errors(fake) {
            return Some(format!("{name} error lines: real {:?}, fake {:?}", errors(real), errors(fake)));
        }
    }
    if real.exit.as_ref() != "0" && failure(case, real) != failure(case, fake) {
        return Some(format!("failure tail: real {:?}, fake {:?}", failure(case, real), failure(case, fake)));
    }
    if list_kinds(real) != list_kinds(fake) {
        return Some("list names or kinds".into());
    }
    if let Some(stanzas) = stanzas
        && kept(stanzas) != kept(fake.stanzas.as_deref().unwrap_or_default())
    {
        return Some("Inst stanzas' KEPT fields".into());
    }
    None
}
