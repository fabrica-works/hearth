//! Startup paths and scenarios shared by the simulated builder checks.
use super::{super::MOTD, BUILDER, Call, File, GIT_DIR, STORE, Scenario, Shared, file_world, resolving, thin};
use hearth_fake_machine::linux::{self, Op, Pid, Reply};
use skein_lib::{Duration, Time};
use skein_world::{StartupAppend, StartupRoot};

/// The named seed used by each focused simulator story.
pub const SIMULATION_DEADLINE: Time = Time::from_nanos(1_000_000_000_000);
pub const SIMULATION_SEED: u64 = file_world::FILE_SEED;
pub const OUTPUT: &[u8] = b"/srv/calls/output";
pub const ERROR: &[u8] = b"/srv/calls/error";
pub const SECOND_OUTPUT: &[u8] = b"/srv/calls/second-output";
pub const SECOND_ERROR: &[u8] = b"/srv/calls/second-error";
/// A held store lock outlives render's ten-minute resolution timer.
pub const LONG_LOCK_RELEASE: Time = Time::from_nanos(700_000_000_000);
pub const RESOLVE_IDLE: Duration = hearth_protocol_builder::limits::SHIPPED.mmdebstrap.idle;
pub const WAITING_TEXT: &[u8] = b"waiting for the store's lock: /srv/store\n";
pub const CHANGED_PACKAGE_TARGET: File = File {
    path: "hosts/x1/x1-prod.json",
    body: br#"{"placement":{"kind":"metal","firmware":"efi","hardware":"7th-gen"},"motd":{},"extra":{}}"#,
    mode: 0o644,
};
pub const DIFF_PACKAGE_TASK: File =
    File { path: "tasks/extra/task.json", body: br#"{"packages":{"pkg-23":{}}}"#, mode: 0o644 };
#[must_use]
pub fn diff_resolving() -> Scenario<'static> {
    Scenario {
        files: MOTD,
        call: Call::Diff {
            resolve: true,
            repository: GIT_DIR.into(),
            from: b"before".as_slice().into(),
            to: b"HEAD".as_slice().into(),
            targets: Box::new([b"x1-lab".as_slice().into(), b"x1-prod".as_slice().into()]),
        },
        form: hearth::Form::JsonLines,
        ..thin()
    }
}
#[must_use]
pub fn startup_roots() -> Vec<StartupRoot> {
    [
        (b"store".as_slice(), STORE),
        (b"cache", b"/srv/store/cache"),
        (b"resolutions", b"/srv/store/resolutions"),
        (b"staging", b"/srv/store/staging"),
    ]
    .into_iter()
    .map(|(name, path)| StartupRoot { name: name.into(), path: path.into() })
    .collect()
}
#[must_use]
pub fn startup_appends(second: bool) -> Vec<StartupAppend> {
    let (output, error) = if second {
        (b"second-output".as_slice(), b"second-error".as_slice())
    } else {
        (b"output".as_slice(), b"error".as_slice())
    };
    [(b"output".as_slice(), output), (b"error".as_slice(), error)]
        .into_iter()
        .map(|(name, path)| StartupAppend {
            name: name.into(),
            root: b"/srv/calls".as_slice().into(),
            path: path.into(),
            mode: 0o600,
        })
        .collect()
}
#[must_use]
pub fn tool_roots(_spawn: &skein_io::kernel::Spawn) -> Vec<StartupRoot> {
    vec![
        StartupRoot { name: b"root".as_slice().into(), path: b"/".as_slice().into() },
        StartupRoot { name: b"cwd".as_slice().into(), path: STORE.into() },
    ]
}
pub fn lay_calls(shared: &Shared) {
    shared
        .borrow_mut()
        .behind
        .machine_mut()
        .linux_mut()
        .lay(&[linux::Item::directory(b"/srv/calls", 0o700).owner(BUILDER)]);
}
#[must_use]
pub fn simulation_records(shared: &Shared) -> Vec<hearth_protocol_tree::resolution::Resolution> {
    shared
        .borrow()
        .renames
        .iter()
        .filter_map(|rename| {
            if !rename.to.ends_with(b".record") {
                return None;
            }
            let path = [b"/srv/store/resolutions/".as_slice(), &rename.to].concat();
            shared.contents(&path).map(|bytes| {
                hearth_protocol_tree::resolution::read(&bytes, &hearth_protocol_tree::limits::SHIPPED)
                    .expect("resolution record")
            })
        })
        .collect()
}
#[derive(Debug)]
pub struct HeldLock {
    pid: Pid,
    file: linux::Handle,
}
impl HeldLock {
    #[must_use]
    pub fn new(shared: &Shared) -> Self {
        let mut fake = shared.borrow_mut();
        let machine = fake.behind.machine_mut();
        let pid = machine.linux_mut().login(BUILDER, STORE);
        let Reply::Opened(file) = machine
            .call(pid, Op::Open { at: linux::At::Root, path: STORE.into(), how: linux::How::Directory })
            .expect("store root")
        else {
            panic!("opened store")
        };
        machine.call(pid, Op::Lock { file }).expect("external lock held");
        Self { pid, file }
    }
    pub fn release(self, shared: &Shared) {
        let mut fake = shared.borrow_mut();
        let machine = fake.behind.machine_mut();
        machine.call(self.pid, Op::Close { handle: self.file }).expect("release held lock");
        machine.call(self.pid, Op::Exit { exit: linux::Exit::Code(0) }).expect("lock holder exits");
    }
}

#[must_use]
pub fn client_exits(outcome: &skein_world::Outcome<crate::simulated::Proc, Shared>) -> Vec<skein_io::kernel::Exit> {
    use skein_shell::Host;
    outcome
        .procs
        .iter()
        .filter_map(|proc| if matches!(proc, crate::simulated::Proc::Client(_)) { proc.exit() } else { None })
        .collect()
}
/// The stable cache contents; each record's time stays on its world's clock.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct StoreFiles {
    payloads: std::collections::BTreeMap<Vec<u8>, Vec<u8>>,
    records: std::collections::BTreeMap<
        Vec<u8>,
        (hearth_protocol_tree::cache_file::Origin, hearth_lib::encode::Digest, u64),
    >,
}
#[must_use]
pub fn store_files(shared: &Shared) -> StoreFiles {
    let mut fake = shared.borrow_mut();
    let machine = fake.behind.machine_mut();
    let pid = machine.linux_mut().login(BUILDER, STORE);
    let mut snapshot = StoreFiles::default();
    for path in [STORE, b"/srv/store/cache", b"/srv/store/resolutions", b"/srv/store/staging"] {
        let entries = snapshot_directory(machine, pid, path);
        assert!(path != b"/srv/store/staging" || entries.is_empty(), "no staging entry survives");
        for (kind, name) in entries {
            assert!(!name.ends_with(b".tmp") && !name.windows(7).any(|part| part == b".skein-"), "no temporary");
            if kind == linux::Kind::File && path != STORE {
                let name = [path, b"/", &name].concat();
                let bytes = machine.linux().contents(&name).expect("store file");
                if path == b"/srv/store/cache" && name.ends_with(b".record") {
                    let record = hearth_protocol_tree::cache_file::read(bytes, &hearth_protocol_tree::limits::SHIPPED)
                        .expect("cache record");
                    snapshot.records.insert(name, (record.origin, record.sha256, record.size));
                } else {
                    snapshot.payloads.insert(name, bytes.to_vec());
                }
            }
        }
    }
    machine.call(pid, Op::Exit { exit: linux::Exit::Code(0) }).expect("snapshot reader exits");
    snapshot
}
fn snapshot_directory(
    machine: &mut hearth_fake_machine::Machine,
    pid: Pid,
    path: &[u8],
) -> Box<[(linux::Kind, Box<[u8]>)]> {
    let Reply::Opened(dir) = machine
        .call(pid, Op::Open { at: linux::At::Root, path: path.into(), how: linux::How::Directory })
        .expect("snapshot directory")
    else {
        panic!("opened directory")
    };
    let Reply::Listed(entries) = machine.call(pid, Op::List { dir, most: SNAPSHOT_PAGE }).expect("snapshot entries")
    else {
        panic!("listed directory")
    };
    assert!(entries.len() < usize::try_from(SNAPSHOT_PAGE).expect("page fits"), "the whole snapshot fits one page");
    machine.call(pid, Op::Close { handle: dir }).expect("snapshot reader closes directory");
    entries
}
/// Greater than the number of files in the thin store fixture.
const SNAPSHOT_PAGE: u32 = 4096;
#[must_use]
pub fn tool_spans(trace: &[skein_sim::Entry], name: &[u8]) -> Vec<Duration> {
    use skein_io::kernel::Done;
    use skein_sim::{Event, Summary, Text};
    let mut submissions = std::collections::BTreeMap::new();
    let mut children = std::collections::BTreeMap::new();
    let mut spans = Vec::new();
    for entry in trace {
        if let Event::Submit { op, kind: Summary::Spawn { program, .. } } = entry.event
            && program == Text::of(name)
        {
            submissions.insert((entry.pid, op), entry.at);
        }
        if let Event::Complete { op, result: Ok(Done::Spawned { pidfd }), .. } = entry.event
            && let Some(started) = submissions.remove(&(entry.pid, op))
        {
            children.insert((entry.pid, pidfd), started);
        }
        if let Event::Complete { kind: Summary::Wait { pidfd, .. }, result: Ok(Done::Exit(_)), .. } = entry.event
            && let Some(started) = children.remove(&(entry.pid, pidfd))
        {
            spans.push(entry.at.saturating_since(started));
        }
    }
    assert!(children.is_empty(), "every spawned fake exited and was reaped");
    spans
}

/// The pair requests the old empty apt set and the new pkg-23 set.
pub const DIFF_RESOLUTION_RUNS: usize = 2;
#[must_use]
pub fn simulated_thin() -> skein_world::Outcome<crate::simulated::Proc, Shared> {
    crate::simulated::run(&thin(), SIMULATION_SEED, skein_sim::Config::calm())
}
#[must_use]
pub fn simulated_resolved() -> skein_world::Outcome<crate::simulated::Proc, Shared> {
    crate::simulated::run(&resolving(super::super::THIN), SIMULATION_SEED, skein_sim::Config::calm())
}
#[must_use]
pub fn pace_bounds(outcome: &skein_world::Outcome<crate::simulated::Proc, Shared>) -> (Duration, Duration) {
    outcome
        .procs
        .iter()
        .filter_map(|proc| match proc {
            crate::simulated::Proc::Client(_) => None,
            crate::simulated::Proc::Git(tool) | crate::simulated::Proc::Mmdebstrap(tool) => {
                Some(tool.work_bounds(crate::pace::Pace::calm()))
            }
        })
        .fold((Duration::ZERO, Duration::ZERO), |(min, max), (lower, upper)| {
            (min.saturating_add(lower), max.saturating_add(upper))
        })
}
/// Stalling before the first output byte leaves the idle timer as the only progress.
pub const STALL_AFTER: u64 = 0;
/// One target changes its package request; the other target is unchanged.
pub const KNOWN_REBUILDS: usize = 1;
pub const BUSY: linux::Refusal = linux::Refusal::Busy;

/// The replay proof's seed and a neighbouring independent seed.
pub const REPLAY_SEED: u64 = 11;
pub const OTHER_REPLAY_SEED: u64 = REPLAY_SEED + 1;
/// Admission gives the builder the world's first process id.
pub const CLIENT_PID: u32 = 0;
#[must_use]
pub fn digested(seed: u64, config: skein_sim::Config) -> skein_world::Outcome<crate::simulated::Proc, Shared> {
    let mut world = crate::simulated::world(&thin(), seed, config);
    world.digests(skein_world::Digests::Traced);
    world.run()
}
#[must_use]
pub fn replay(seed: u64) -> (Vec<String>, skein_world::StateDigests) {
    let outcome = digested(seed, crate::simulated::chaos());
    (outcome.trace.iter().map(|entry| format!("{entry:?}")).collect(), outcome.state_digests())
}

/// The store's shipped page and one additional file force a subsequent page.
pub const STORE_PAGE: u32 = hearth::limits().domain.store.page;
pub const CACHE_FILES: u32 = STORE_PAGE + 1;
/// The calm client's submission count, checked by the cut sweep (ruling 188).
pub const CUTS: u32 = 143;
pub const BUILDER_SLOT: usize = 0;
pub const FUZZY_SEEDS: u64 = 64;
#[must_use]
pub fn small_limits() -> hearth::Limits {
    let mut limits = hearth::limits();
    limits.protocol.git.entries = 64;
    limits.protocol.git.path = 256;
    limits.protocol.git.paths_total = limits.protocol.git.entries * limits.protocol.git.path;
    limits
}
pub fn lay_cache_pages(shared: &Shared) {
    let mut fake = shared.borrow_mut();
    for file in 0..CACHE_FILES {
        let path = format!("/srv/store/cache/{file:064x}");
        fake.behind.machine_mut().linux_mut().lay(&[linux::Item::file(path.as_bytes(), 0o600, b"").owner(BUILDER)]);
    }
}
#[must_use]
pub fn lists(shared: &Shared) -> u32 {
    shared.borrow().lists
}

/// The thin call's literal output, shared by the simulator's stories.
pub const THIN_OUTPUT: Option<&[u8]> = Some(super::super::THIN_TEXT.as_bytes());

/// One original tool and one cold replacement after the single cut.
pub const CUT_INSTANCES: u32 = 1 + 1;
#[must_use]
pub fn client_submissions(outcome: &skein_world::Outcome<crate::simulated::Proc, Shared>) -> u32 {
    let counts: Vec<_> = outcome
        .procs
        .iter()
        .zip(&outcome.submissions)
        .filter_map(|(proc, count)| matches!(proc, crate::simulated::Proc::Client(_)).then_some(*count))
        .collect();
    *super::only(&counts)
}
