//! The common fake machine, repositories and calls of the builder's worlds.
use super::{File, Laid, THIN};
use core::cell::{Ref, RefCell, RefMut};
use hearth_domain_builder::vocabulary::Call;
use hearth_fake_git::{objects, repository::Repositories};
use hearth_fake_machine::{
    behind::{self, Behind},
    hearth::{draw::Draw, universe::Universe},
    linux,
};
use skein_io::kernel::Error;
use skein_lib::{Queue, Rng};
use skein_sim::{Ask, Handle, Reply};
use std::{collections::BTreeSet, rc::Rc};

pub const WORLD_OWNER: skein_lib::Token = skein_lib::Token::new(900);
pub const BUILDER: linux::Ids = linux::Ids { user: 1001, group: 1001 };
pub const GIT_DIR: &[u8] = b"/srv/repo.git";
pub const STORE: &[u8] = b"/srv/store";
pub const WORLD_SEEDS: core::ops::Range<u64> = 0..8;
pub const WORLD_PACKAGES: u32 = 24;

#[derive(Debug)]
pub struct Rename {
    pub from: Box<[u8]>,
    pub to: Box<[u8]>,
}
#[derive(Debug)]
pub struct Fake {
    pub(super) lists: u32,
    pub behind: Behind,
    pub repositories: Repositories,
    pub universe: Universe,
    pub ledger: Vec<linux::Entry>,
    pub renames: Vec<Rename>,
    pub locks: BTreeSet<Handle>,
    pub served_bytes: u64,
}
#[derive(Clone, Debug)]
pub struct Shared(Rc<RefCell<Fake>>);
impl Shared {
    #[must_use]
    pub fn new(files: &[File], seed: u64) -> Self {
        Self::build(files, &[], seed, WORLD_PIPE, None)
    }
    fn build(files: &[File], laid: &[Laid], seed: u64, pipe: u32, fault: Option<Fault>) -> Self {
        let mut machine = hearth_fake_machine::Machine::new(
            seed,
            linux::Sizes { files: 4096, bytes: 16 << 20, processes: 32, pipe, ledger: 256 },
        );
        machine.linux_mut().lay(&[
            linux::Item::directory(b"/srv", 0o755),
            linux::Item::directory(STORE, 0o700).owner(BUILDER),
            linux::Item::directory(b"/srv/store/cache", 0o700).owner(BUILDER),
            linux::Item::directory(b"/srv/store/resolutions", 0o700).owner(BUILDER),
            linux::Item::directory(b"/srv/store/staging", 0o700).owner(BUILDER),
            linux::Item::directory(b"/usr", 0o755),
            linux::Item::directory(b"/usr/bin", 0o755),
        ]);
        for (name, bytes) in super::laid_files(laid) {
            let path = [b"/srv/store/".as_slice(), name.as_bytes()].concat();
            machine.linux_mut().lay(&[linux::Item::file(&path, 0o600, &bytes).owner(BUILDER)]);
        }
        match fault {
            Some(Fault::NotInstalled { absent: true }) => {}
            Some(Fault::NotInstalled { absent: false }) => {
                machine.linux_mut().lay(&[linux::Item::file(b"/usr/bin/git", 0o644, b"")]);
            }
            _ => {
                machine.install(b"/usr/bin/git", hearth_fake_machine::hearth::Tool::Git);
            }
        }
        machine.install(b"/usr/bin/mmdebstrap", hearth_fake_machine::hearth::Tool::Mmdebstrap);
        let mut repository = hearth_fake_git::repository::Repository::new(b"refs/heads/main");
        repository.commit(&commit(files, Box::default()));
        let mut repositories = Repositories::default();
        repositories.lay(machine.linux_mut(), GIT_DIR, BUILDER, repository);
        let universe = Universe::draw(&mut Rng::new(seed), &Draw { packages: WORLD_PACKAGES, names: &[] });
        Self(Rc::new(RefCell::new(Fake {
            behind: Behind::new(machine, BUILDER),
            repositories,
            universe,
            ledger: Vec::new(),
            renames: Vec::new(),
            locks: BTreeSet::new(),
            served_bytes: 0,
            lists: 0,
        })))
    }
    #[must_use]
    pub fn borrow(&self) -> Ref<'_, Fake> {
        self.0.borrow()
    }
    #[must_use]
    pub fn borrow_mut(&self) -> RefMut<'_, Fake> {
        self.0.borrow_mut()
    }
    #[must_use]
    pub fn served_bytes(&self) -> u64 {
        self.borrow().served_bytes
    }
    #[must_use]
    pub fn contents(&self, path: &[u8]) -> Option<Vec<u8>> {
        self.borrow().behind.machine().linux().contents(path).map(<[u8]>::to_vec)
    }
    pub fn answer(&self, ask: Ask) -> Result<Reply, Error> {
        let rename = if let Ask::Rename { from, to, .. } = &ask {
            Some(Rename { from: from.clone(), to: to.clone() })
        } else {
            None
        };
        let lock = if let Ask::Lock { file } = &ask { Some(*file) } else { None };
        let close = if let Ask::Close { file } = &ask { Some(*file) } else { None };
        let mut fake = self.borrow_mut();
        let listing = matches!(ask, Ask::List { .. });
        let result = behind::answer(&mut fake.behind, ask);
        let entries: Vec<_> = fake.behind.machine_mut().linux_mut().ledger().collect();
        if listing {
            fake.lists = fake.lists.saturating_add(u32::try_from(entries.len()).expect("bounded ledger"));
        }
        fake.ledger.extend(entries);
        if result.is_ok() {
            if let Some(rename) = rename {
                fake.renames.push(rename);
            }
            if let Some(file) = lock {
                fake.locks.insert(file);
            }
            if let Some(file) = close {
                fake.locks.remove(&file);
            }
            assert!(fake.locks.len() <= 1, "testing.md, 8.4: at most one store lock holder");
        }
        result
    }
}
impl skein_world::Machine for Shared {
    fn state_digest(&self) -> Option<[u8; 32]> {
        self.borrow().behind.state_digest()
    }
    fn open_root(&mut self, path: &[u8]) -> Result<Handle, Error> {
        self.borrow_mut().behind.open_root(path)
    }
    fn close_root(&mut self, root: Handle) {
        self.borrow_mut().behind.close_root(root);
    }
    fn open_append(&mut self, root: Handle, path: &[u8], mode: u32) -> Result<Handle, Error> {
        self.borrow_mut().behind.open_append(root, path, mode)
    }
    fn close_append(&mut self, file: Handle) {
        self.borrow_mut().behind.close_append(file);
    }
    fn cut(&mut self, cut: skein_world::Cut, held: &[Handle], seed: u64) {
        let mut fake = self.borrow_mut();
        fake.behind.cut(cut, held, seed);
        fake.locks.retain(|file| !held.contains(file));
    }
    fn step(&mut self, call: skein_sim::Call, answers: &mut Queue<skein_sim::Answer>) {
        answers.push(skein_sim::Answer { ticket: call.ticket, result: self.answer(call.ask) });
    }
}
#[derive(Clone, Debug)]
pub struct Scenario<'a> {
    pub files: &'a [File],
    pub laid: &'a [Laid],
    pub call: Call,
    pub form: hearth::Form,
    pub fault: Option<Fault>,
    pub pace: crate::pace::Pace,
}
#[derive(Clone, Copy, Debug)]
pub enum Fault {
    KilledAt { stdout_bytes: u32 },
    CacheFileGone { sha256: hearth_lib::encode::Digest },
    NotInstalled { absent: bool },
    Stalled { after: u64 },
}
#[must_use]
pub fn thin() -> Scenario<'static> {
    Scenario {
        files: THIN,
        laid: &[],
        call: Call::Render {
            resolve: false,
            repository: GIT_DIR.into(),
            revision: b"HEAD".as_slice().into(),
            targets: Box::default(),
        },
        form: hearth::Form::Text,
        fault: None,
        pace: crate::pace::Pace::calm(),
    }
}
#[must_use]
pub fn resolving(files: &[File]) -> Scenario<'_> {
    Scenario {
        files,
        call: Call::Render {
            resolve: true,
            repository: GIT_DIR.into(),
            revision: b"HEAD".as_slice().into(),
            targets: Box::default(),
        },
        ..thin()
    }
}
#[must_use]
pub fn only<T>(values: &[T]) -> &T {
    assert_eq!(values.len(), 1, "exactly one value");
    values.iter().next().expect("one value")
}

pub const DUPLICATE_BLOB: File = File { path: "tasks/motd/tree/etc/issue", body: b"Welcome to x1.\n", mode: 0o644 };
pub const WITH_DEPENDENCIES: File =
    File { path: "tasks/motd/task.json", body: br#"{"packages":{"pkg-23":{}}}"#, mode: 0o644 };
/// The same scenario is laid afresh under eight independently drawn schedules.
pub fn each_domain(test: impl FnMut(crate::domain::Run, u64)) {
    each_scenario(&thin(), test);
}
pub fn each_scenario(scenario: &Scenario<'_>, mut test: impl FnMut(crate::domain::Run, u64)) {
    for seed in WORLD_SEEDS {
        test(crate::domain::Run::new(scenario, seed), seed);
    }
}
#[must_use]
pub fn spec(run: &crate::domain::Run) -> &hearth_domain_render::parts::Spec {
    let hearth_domain_builder::vocabulary::Answer::Spec(spec) = only(&run.answers) else {
        unreachable!("one spec answered")
    };
    spec
}
#[must_use]
pub fn recorded(run: &crate::domain::Run) -> hearth_protocol_tree::resolution::Resolution {
    let spec = spec(run);
    let resolution = spec.resolution.as_deref().expect("resolution requested");
    let name = format!("/srv/store/resolutions/{}.record", super::hex_digest(resolution.key().0));
    hearth_protocol_tree::resolution::read(
        &run.shared.contents(name.as_bytes()).expect("recorded closure"),
        &hearth_protocol_tree::limits::SHIPPED,
    )
    .expect("valid resolution record")
}
pub const LOCK_RELEASE: skein_lib::Time = skein_lib::Time::from_nanos(1_000_000_000);
pub const LOCK_RETRY: skein_lib::Duration = hearth_domain_store::limits::SHIPPED.retry;

/// A drawn DAG with one alternative per group for the independent closure oracle.
#[must_use]
pub fn first_alternatives(run: crate::domain::Run) -> crate::domain::Run {
    let packages = run
        .shared
        .borrow()
        .universe
        .packages()
        .iter()
        .cloned()
        .map(|mut package| {
            for group in package.depends.iter_mut().chain(package.pre_depends.iter_mut()) {
                *group = group.iter().take(1).cloned().collect();
            }
            package
        })
        .collect();
    run.shared.borrow_mut().universe = Universe::new(packages);
    run
}
pub mod file_world;

/// A completed domain scenario, laid and settled once for each schedule seed.
pub fn each_settled(scenario: &Scenario<'_>, mut test: impl FnMut(crate::domain::Run, u64)) {
    each_scenario(scenario, |run, seed| test(run.finish(), seed));
}

pub const THIN_MOTD_SHA256_HEX: &str = "775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123";

/// Pipe capacity normally shipped by the fake machine, and the named small-pipe exception.
pub const WORLD_PIPE: u32 = 4096;
/// Store, cache, resolutions and staging remain open until the world exits.
pub const WORLD_ROOTS: usize = 4;
/// The plan's pipe exception forces ls-tree to stop before its next entry.
pub const SMALL_PIPE: u32 = 64;
/// Eight two-record domain output groups fit before the next service pass.
pub const PROTOCOL_QUEUE: u32 = 16;
/// Output identities live above file tokens and below child/pipe identities.
pub const PROTOCOL_OUTPUT: skein_lib::Token = skein_lib::Token::new(10_000);
/// The next output identity is stderr's independent stream.
pub const PROTOCOL_ERROR: skein_lib::Token = skein_lib::Token::new(10_001);
/// Child and pipe identities start above the two output streams.
pub const PROTOCOL_ENTITY: u64 = 20_000;
/// Fail fast after twenty thousand iterations if the world cannot settle.
pub const PROTOCOL_TURNS: u32 = 20_000;
/// The plan's Wire grain bounds each machine Take to thirty bytes.
pub const PIPE_GRAIN: u64 = 30;
/// File and io terminals use the domain world's one-to-twenty-ms span.
pub const PROTOCOL_LATENCY: skein_world::domain::Span = skein_world::domain::Span::millis(1, 20);
/// One millisecond advances a live child between its bounded turns.
pub const PROTOCOL_TICK: skein_lib::Duration = skein_lib::Duration::from_millis(1);

fn commit(files: &[File], parents: Box<[objects::Oid]>) -> objects::Commit {
    let mut tree = objects::Tree::new();
    for file in files {
        let mode = match file.mode {
            0o120_000 => objects::Mode::Link,
            0o755 => objects::Mode::Executable,
            _ => objects::Mode::File,
        };
        tree.insert(file.path.as_bytes(), objects::Entry::Blob { mode, bytes: file.body.into() });
    }
    let signature = objects::Signature {
        name: b"hearth".as_slice().into(),
        email: b"hearth@hearth.invalid".as_slice().into(),
        seconds: 1_700_000_000,
        zone: *b"+0000",
    };
    objects::Commit {
        tree,
        parents,
        author: signature.clone(),
        committer: signature,
        message: b"thin\n".as_slice().into(),
    }
}

impl Shared {
    #[must_use]
    pub fn for_scenario(scenario: &Scenario<'_>, seed: u64, pipe: u32) -> Self {
        let shared = Self::build(scenario.files, scenario.laid, seed, pipe, scenario.fault);
        if let Call::Diff { resolve, .. } = scenario.call {
            let mut fake = shared.borrow_mut();
            let repository = fake.repositories.0.get_mut(GIT_DIR).expect("laid repository");
            let hearth_fake_git::repository::Resolved::Commit(before) = repository.resolve(b"HEAD") else {
                unreachable!("initial commit")
            };
            repository.set_ref(b"refs/heads/before", before);
            let replacement = if resolve {
                CHANGED_PACKAGE_TARGET
            } else if scenario.files.iter().any(|file| file.path == super::MOTD_KIND.path) {
                super::MOTD_KIND
            } else {
                super::budgets::FLEET_EDIT
            };
            let mut files: Vec<_> = scenario
                .files
                .iter()
                .map(|file| if file.path == replacement.path { replacement } else { *file })
                .collect();
            if resolve {
                files.push(DIFF_PACKAGE_TASK);
            }
            repository.commit(&commit(&files, Box::new([before])));
        }
        shared
    }
    /// Preserve the machine ledger after every pure fake turn as after every file record.
    pub fn drain_ledger(&self) {
        let mut fake = self.borrow_mut();
        let entries: Vec<_> = fake.behind.machine_mut().linux_mut().ledger().collect();
        fake.ledger.extend(entries);
    }
}

mod protocol_world;
pub use protocol_world::*;

/// Hosted tools reserve one in-flight kernel operation.
pub const HOST_OPERATIONS: u32 = 1;
/// Read files and stdin in the same four-KiB chunks as the protocol world's pipes.
pub const HOST_BUFFER: usize = WORLD_PIPE as usize;
/// The plan's one-MiB bound for each hosted tool.
pub const FAKE_WORST_CASE: u64 = 1 << 20;

/// The adapter's one startup-close identity cannot collide with service operations.
pub const STARTUP_CLOSE: skein_lib::Token = skein_lib::Token::new(u64::MAX);

mod simulation_world;
pub use simulation_world::*;
