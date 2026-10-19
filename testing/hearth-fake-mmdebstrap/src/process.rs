//! Simulate as a process, with the child's files, permissions and pipes (domain/testing.md, 4.1).
use crate::{
    face::{self, Inst, Outcome, Printed, Simulate, Source},
    resolve::{self, Request, Selected, Variant},
};
use hearth_fake_machine::{
    Machine,
    hearth::{
        Tool,
        mirror::{Mirror, Release},
        universe::Universe,
    },
    linux::{At, Exit, Handle, How, Op, Pid, Refusal, Reply, Signal},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Faults {
    pub stall_after: Option<u64>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Progress {
    Working,
    Waiting,
    Exited,
}
#[derive(Debug)]
pub struct Mmdebstrap {
    child: Pid,
    printed: Printed,
    offsets: [usize; 2],
    faults: Faults,
    sent: u64,
    exited: bool,
}
impl Mmdebstrap {
    pub fn start(machine: &mut Machine, child: Pid, faults: Faults) -> Self {
        assert_eq!(machine.tool(child), Some(Tool::Mmdebstrap));
        let invocation = machine.linux().invocation(child);
        assert_eq!(invocation.program.as_ref(), b"/usr/bin/mmdebstrap");
        let simulate = face::check(&invocation.args, &invocation.env);
        let target = [simulate.scratch.as_ref(), b"/root"].concat();
        assert_eq!(
            open(machine, child, &target, How::Directory),
            Err(Refusal::NotFound),
            "a new scratch root for each run"
        );
        let text = read(machine, child, &[simulate.scratch.as_ref(), b"/sources.list"].concat());
        let sources = face::sources(&text);
        let scratch = open(machine, child, &simulate.scratch, How::Directory).expect("scratch directory");
        for path in [
            b"root".as_slice(),
            b"root/var",
            b"root/var/lib",
            b"root/var/lib/apt",
            b"root/var/lib/apt/lists",
            b"root/var/lib/apt/lists/partial",
        ] {
            machine
                .call(child, Op::MakeDirectory { dir: scratch, path: path.into(), mode: 0o755 })
                .expect("make apt directories as the child");
        }
        let lists = [target.as_slice(), b"/var/lib/apt/lists/"].concat();
        write(machine, child, &[lists.as_slice(), b"lock"].concat(), b"");
        let (files, outcome) = match machine.reach(simulate.proxy) {
            None => (Vec::new(), Outcome::Unreachable),
            Some(mirror) => resolve(mirror, &simulate, &sources),
        };
        if !matches!(outcome, Outcome::Resolved(_)) {
            machine
                .call(
                    child,
                    Op::MakeDirectory {
                        dir: scratch,
                        path: b"root/var/lib/apt/lists/auxfiles".as_slice().into(),
                        mode: 0o755,
                    },
                )
                .expect("apt's failed pass leaves auxfiles");
        }
        machine.call(child, Op::Close { handle: scratch }).expect("close scratch");
        for (name, bytes) in files {
            write(machine, child, &[lists.as_slice(), &name].concat(), &bytes);
        }
        Self {
            child,
            printed: face::print(&simulate, &sources, &outcome),
            offsets: [0, 0],
            faults,
            sent: 0,
            exited: false,
        }
    }
    fn exit(&mut self, machine: &mut Machine, exit: Exit) -> Progress {
        machine.call(self.child, Op::Exit { exit }).expect("child exits");
        self.exited = true;
        Progress::Exited
    }
    pub fn step(&mut self, machine: &mut Machine) -> Progress {
        if self.exited {
            return Progress::Exited;
        }
        if machine.linux().pending(self.child) == Some(Signal::Terminate) {
            return self.exit(machine, Exit::Signal(15));
        }
        for place in 0..2 {
            let bytes = if place == 0 { &self.printed.stdout } else { &self.printed.stderr };
            let offset = &mut self.offsets[place];
            if *offset == bytes.len() {
                continue;
            }
            let room = self
                .faults
                .stall_after
                .map_or(usize::MAX, |limit| usize::try_from(limit.saturating_sub(self.sent)).unwrap_or(usize::MAX));
            if room == 0 {
                return Progress::Waiting;
            }
            let end = bytes.len().min(offset.saturating_add(room));
            let pipe = machine.linux().descriptor(self.child, u32::try_from(place + 1).expect("stdout or stderr"));
            match machine.call(self.child, Op::Send { pipe, bytes: bytes[*offset..end].into() }) {
                Ok(Reply::Sent(count)) => {
                    *offset += usize::try_from(count).expect("pipe count fits");
                    self.sent += u64::from(count);
                    return if count == 0 { Progress::Waiting } else { Progress::Working };
                }
                Err(Refusal::BrokenPipe) => return self.exit(machine, Exit::Signal(13)),
                reply => panic!("pipe send answers bytes or EPIPE: {reply:?}"),
            }
        }
        self.exit(machine, Exit::Code(self.printed.exit))
    }
}
type Lists = Vec<(Box<[u8]>, Box<[u8]>)>;
fn resolve(mirror: &Mirror, simulate: &Simulate, sources: &[Source]) -> (Lists, Outcome) {
    let mut files = Vec::new();
    let mut universes = Vec::new();
    let mut releases = Vec::new();
    for (line, source) in sources.iter().enumerate() {
        let Some(suite) = mirror.suite(&source.archive, source.at, &source.suite) else {
            return (files, Outcome::NotFound { line: u32::try_from(line + 1).expect("source line fits") });
        };
        files.push((face::list_name(source, b"InRelease"), suite.in_release().into()));
        for component in &source.components {
            let universe = suite.universe(component).expect("source component served");
            universes.push(universe);
            releases.push(suite.release());
            let path = [component.as_ref(), b"/binary-", &simulate.architecture, b"/Packages"].concat();
            files.push((face::list_name(source, &path), suite.index(component).expect("source index").into()));
        }
    }
    let request = Request { universes: &universes, variant: simulate.variant, include: &simulate.include };
    let outcome = match resolve::closure(&request) {
        Ok(selected) => Outcome::Resolved(insts(&universes, &releases, &selected)),
        Err(mut failure) => {
            let essential =
                resolve::closure(&Request { universes: &universes, variant: Variant::Essential, include: &[] })
                    .expect("essential set resolves before the apt pass");
            let mut providers = Vec::new();
            if let resolve::Failure::NoCandidate(name) = &failure {
                for (source, universe) in universes.iter().enumerate() {
                    providers.extend(universe.providers(name).iter().map(|package| Selected {
                        universe: u32::try_from(source).expect("source fits"),
                        package: *package,
                    }));
                }
            }
            if let resolve::Failure::Unsatisfiable(broken) = &mut failure {
                for broken in broken {
                    let name = broken.relation.split(|b| *b == b' ').next().expect("relation name");
                    if !universes.iter().any(|universe| universe.package(name).is_some()) {
                        broken.against = None;
                    }
                }
            }
            Outcome::Failed {
                essential: insts(&universes, &releases, &essential),
                providers: insts(&universes, &releases, &providers),
                failure,
            }
        }
    };
    (files, outcome)
}
fn insts(universes: &[&Universe], releases: &[&Release], selected: &[Selected]) -> Box<[Inst]> {
    selected
        .iter()
        .map(|selected| {
            let package = &universes[usize::try_from(selected.universe).expect("source fits")].packages()
                [usize::try_from(selected.package).expect("package fits")];
            let mut origins = Vec::new();
            for (universe, release) in universes.iter().zip(releases) {
                if universe.package(&package.name).is_some_and(|other| other.version == package.version) {
                    if !origins.is_empty() {
                        origins.extend_from_slice(b", ");
                    }
                    origins.extend_from_slice(&release.origin);
                    origins.push(b':');
                    if let Some(version) = &release.version {
                        origins.extend_from_slice(version);
                        origins.push(b'/');
                    }
                    origins.extend_from_slice(&release.suite);
                }
            }
            Inst {
                name: package.name.clone(),
                version: package.version.clone(),
                architecture: package.architecture.clone(),
                origins: origins.into(),
            }
        })
        .collect()
}
fn open(machine: &mut Machine, child: Pid, path: &[u8], how: How) -> Result<Handle, Refusal> {
    let Reply::Opened(handle) = machine.call(child, Op::Open { at: At::Root, path: path.into(), how })? else {
        panic!("open answers a handle");
    };
    Ok(handle)
}
fn read(machine: &mut Machine, child: Pid, path: &[u8]) -> Box<[u8]> {
    let file = open(machine, child, path, How::Read).expect("source list readable by child");
    let mut bytes = Vec::new();
    loop {
        let Reply::Read(part) = machine
            .call(child, Op::Read { file, at: u64::try_from(bytes.len()).expect("source length fits"), most: 4096 })
            .expect("read sources")
        else {
            panic!("read answers bytes");
        };
        if part.is_empty() {
            break;
        }
        bytes.extend_from_slice(&part);
    }
    machine.call(child, Op::Close { handle: file }).expect("close sources");
    bytes.into()
}
fn write(machine: &mut Machine, child: Pid, path: &[u8], bytes: &[u8]) {
    let file = open(machine, child, path, How::Create { mode: None }).expect("create apt list as child");
    machine.call(child, Op::Write { file, at: 0, bytes: bytes.into() }).expect("write apt list");
    machine.call(child, Op::Close { handle: file }).expect("close apt list");
}
