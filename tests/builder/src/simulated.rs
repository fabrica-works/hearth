//! The shipped client and hosted tools over the fake machine (testing.md, 2.6).
mod client;
use crate::{
    fixtures::{self, Scenario, Shared},
    hosted::{self, Hosted},
};
pub use client::Client;
use skein_io::kernel::{Complete, Exit, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;
use skein_sim::Config;
use skein_world::{HostedProgram, Inherited, Memory, Outcome, World};
use std::cell::RefCell;

#[derive(Debug)]
pub enum Proc {
    Client(Box<Client>),
    Git(Box<Hosted>),
    Mmdebstrap(Box<Hosted>),
}
impl Proc {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Client(host) => host.as_ref(),
            Self::Git(host) | Self::Mmdebstrap(host) => host.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Client(host) => host.as_mut(),
            Self::Git(host) | Self::Mmdebstrap(host) => host.as_mut(),
        }
    }
}
impl Host for Proc {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.host_mut().iterate(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host_mut().completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host_mut().submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host().work_pending(now)
    }
    fn next_deadline(&self) -> Option<Time> {
        self.host().next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.host().is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        self.host().exit()
    }
    fn operations(&self) -> u32 {
        self.host().operations()
    }
    fn worst_case(&self) -> u64 {
        self.host().worst_case()
    }
}
#[derive(Clone, Debug)]
struct Context {
    scenario: Scenario<'static>,
    shared: Shared,
    seed: u64,
}
thread_local! { static CONTEXT: RefCell<Option<Context>> = const { RefCell::new(None) }; }
fn context() -> Context {
    CONTEXT.with(|context| context.borrow().as_ref().expect("configured world factories").clone())
}
#[must_use]
pub fn client(inherited: &Inherited) -> Proc {
    let context = context();
    Proc::Client(Box::new(Client::new(inherited, &context.scenario, context.seed)))
}
#[must_use]
pub fn git(spawn: &skein_io::kernel::Spawn, inherited: &Inherited) -> Proc {
    let context = context();
    Proc::Git(Box::new(hosted::git::Git::host(spawn, inherited, &context.scenario, context.shared, context.seed)))
}
#[must_use]
pub fn mmdebstrap(spawn: &skein_io::kernel::Spawn, inherited: &Inherited) -> Proc {
    let context = context();
    Proc::Mmdebstrap(Box::new(hosted::mmdebstrap::Mmdebstrap::host(
        spawn,
        inherited,
        &context.scenario,
        context.shared,
        context.seed,
    )))
}

/// Build a restartable client over the scenario's machine and installed tools.
#[must_use]
pub fn world(scenario: &Scenario<'_>, seed: u64, config: Config) -> World<Proc, Referee, Shared> {
    build(scenario, seed, config, None)
}
fn build(scenario: &Scenario<'_>, seed: u64, config: Config, release: Option<Time>) -> World<Proc, Referee, Shared> {
    let shared = Shared::for_scenario(scenario, seed, fixtures::WORLD_PIPE);
    fixtures::serve(&shared);
    fixtures::lay_calls(&shared);
    let lock = release.map(|at| (fixtures::HeldLock::new(&shared), at));
    CONTEXT.with(|context| {
        *context.borrow_mut() = Some(Context {
            scenario: Scenario {
                files: &[],
                laid: &[],
                call: scenario.call.clone(),
                form: scenario.form,
                fault: scenario.fault,
                pace: scenario.pace,
            },
            shared: shared.clone(),
            seed,
        });
    });
    let mut world = World::new_controlled(seed, config, Memory::Unchecked, |_| Referee {
        shared: shared.clone(),
        done: false,
        lock,
        seen: 0,
    })
    .with_machine(shared);
    world.host_roots(
        HostedProgram {
            program: b"/usr/bin/git".as_slice().into(),
            make: git,
            instances: 1,
            operations: fixtures::HOST_OPERATIONS,
        },
        fixtures::tool_roots,
    );
    world.host_roots(
        HostedProgram {
            program: b"/usr/bin/mmdebstrap".as_slice().into(),
            make: mmdebstrap,
            instances: 1,
            operations: fixtures::HOST_OPERATIONS,
        },
        fixtures::tool_roots,
    );
    world.spawn_restartable_startup(fixtures::startup_roots(), fixtures::startup_appends(false), client);
    world
}
#[must_use]
pub fn run(scenario: &Scenario<'_>, seed: u64, config: Config) -> Outcome<Proc, Shared> {
    world(scenario, seed, config).run()
}
#[must_use]
pub fn two(scenario: &Scenario<'_>, seed: u64, config: Config) -> Outcome<Proc, Shared> {
    let mut world = world(scenario, seed, config);
    world.spawn_restartable_startup(fixtures::startup_roots(), fixtures::startup_appends(true), client);
    world.run()
}
#[must_use]
pub fn with_lock(scenario: &Scenario<'_>, seed: u64, config: Config, release: Time) -> Outcome<Proc, Shared> {
    build(scenario, seed, config, Some(release)).run()
}
#[derive(Debug)]
pub struct Referee {
    shared: Shared,
    done: bool,
    lock: Option<(fixtures::HeldLock, Time)>,
    seen: usize,
}
impl skein_world::Referee<Proc> for Referee {
    fn act(&mut self, now: Time, _procs: &mut [Proc]) {
        if self.lock.as_ref().is_some_and(|(_, at)| now >= *at) {
            self.lock.take().expect("held lock").0.release(&self.shared);
        }
    }
    fn observe(&mut self, _now: Time, procs: &[Proc]) {
        let mmdebstraps: Vec<_> = procs.iter().filter(|proc| matches!(proc, Proc::Mmdebstrap(_))).collect();
        assert!(mmdebstraps.iter().filter(|proc| !proc.is_empty()).count() <= 1, "one mmdebstrap alive at a time");
        if mmdebstraps.len() > self.seen {
            assert_eq!(
                self.shared.contents(fixtures::OUTPUT).as_deref(),
                Some(b"".as_slice()),
                "resolve finishes before the client prints"
            );
            assert_eq!(
                fixtures::simulation_records(&self.shared).len(),
                self.seen,
                "earlier runs are recorded before the next spawn"
            );
            self.seen = mmdebstraps.len();
        }
        self.done = procs.iter().filter(|proc| matches!(proc, Proc::Client(_))).all(Host::is_empty);
    }
    fn next_deadline(&self) -> Option<Time> {
        if let Some((_, at)) = self.lock.as_ref() {
            Some(*at)
        } else {
            (!self.done).then_some(fixtures::SIMULATION_DEADLINE)
        }
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.done && now >= fixtures::SIMULATION_DEADLINE).then(|| "the simulated builder must settle".to_owned())
    }
    fn passed(&self) -> bool {
        self.done
    }
}
