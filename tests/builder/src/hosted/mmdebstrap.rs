//! Apt's pure resolver and printer, with its lists written through the kernel.
use super::{Hosted, Job, Tool};
use crate::fixtures::{Scenario, Shared};
use hearth_fake_git::face::{Read, Want};
use hearth_fake_machine::hearth::{
    mirror::{Mirror, Release},
    universe::Universe,
};
use hearth_fake_mmdebstrap::{
    face::{self, Inst, Outcome, Printed, Simulate, Source},
    resolve::{self, Request, Selected, Variant},
};
use skein_io::kernel::Spawn;
use skein_lib::Duration;
use skein_world::Inherited;
use std::collections::{BTreeMap, VecDeque};
#[derive(Debug)]
pub struct Mmdebstrap {
    simulate: Simulate,
    printed: Option<Printed>,
    per_package: Duration,
}
impl Mmdebstrap {
    #[must_use]
    pub fn host(spawn: &Spawn, inherited: &Inherited, scenario: &Scenario<'_>, shared: Shared, seed: u64) -> Hosted {
        let simulate = face::check(&spawn.args, &spawn.env);
        let jobs = VecDeque::from([
            Job::Verify(spawn.program.clone()),
            Job::Read([simulate.scratch.as_ref(), b"/sources.list"].concat().into()),
        ]);
        let delay = scenario.pace.mmdebstrap(&mut super::rng(seed, spawn), 0);
        Hosted::new(
            Tool::Mmdebstrap(Self { simulate, printed: None, per_package: scenario.pace.per_package }),
            shared,
            inherited,
            jobs,
            delay,
            scenario.fault,
        )
    }
    pub(super) fn prepare(
        &mut self,
        shared: &Shared,
        files: &mut BTreeMap<Box<[u8]>, Read>,
        jobs: &mut VecDeque<Job>,
    ) -> Duration {
        let sources_path = [self.simulate.scratch.as_ref(), b"/sources.list"].concat();
        let Read::Contents { bytes, .. } = files.remove(sources_path.as_slice()).expect("sources read") else {
            panic!("sources readable")
        };
        let sources = face::sources(&bytes);
        let (lists, outcome) = {
            let fake = shared.borrow();
            fake.behind
                .machine()
                .reach(self.simulate.proxy)
                .map_or_else(|| (Vec::new(), Outcome::Unreachable), |mirror| resolve(mirror, &self.simulate, &sources))
        };
        let fetched = lists.iter().map(|(_, bytes)| u64::try_from(bytes.len()).expect("list length")).sum::<u64>();
        shared.borrow_mut().served_bytes += fetched;
        let target = [self.simulate.scratch.as_ref(), b"/root"].concat();
        for suffix in [
            b"".as_slice(),
            b"/var",
            b"/var/lib",
            b"/var/lib/apt",
            b"/var/lib/apt/lists",
            b"/var/lib/apt/lists/partial",
        ] {
            jobs.push_back(Job::Make([target.as_slice(), suffix].concat().into()));
        }
        let directory = [target.as_slice(), b"/var/lib/apt/lists/"].concat();
        jobs.push_back(Job::Write([directory.as_slice(), b"lock"].concat().into(), Box::default()));
        if !matches!(outcome, Outcome::Resolved(_)) {
            jobs.push_back(Job::Make([directory.as_slice(), b"auxfiles"].concat().into()));
        }
        for (name, bytes) in lists {
            jobs.push_back(Job::Write([directory.as_slice(), &name].concat().into(), bytes));
        }
        let count = match &outcome {
            Outcome::Resolved(packages) => packages.len(),
            Outcome::Unreachable | Outcome::NotFound { .. } | Outcome::Failed { .. } => 0,
        };
        let mut printed = face::print(&self.simulate, &sources, &outcome);
        printed.stderr = [format!("Fetched {fetched} B in 0s\n").as_bytes(), &printed.stderr].concat().into();
        self.printed = Some(printed);
        Duration::from_nanos(self.per_package.as_nanos().saturating_mul(u64::try_from(count).expect("package count")))
    }
    pub(super) fn next(&mut self) -> Want {
        let printed = self.printed.as_mut().expect("prepared mmdebstrap");
        if !printed.stdout.is_empty() {
            return Want::Stdout(std::mem::take(&mut printed.stdout));
        }
        if !printed.stderr.is_empty() {
            return Want::Stderr(std::mem::take(&mut printed.stderr));
        }
        Want::Exit(printed.exit)
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
