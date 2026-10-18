//! Git's pure byte face with filesystem reads through hosted kernel operations.
use super::{Hosted, Job, Tool};
use crate::fixtures::{Scenario, Shared};
use hearth_fake_git::face::{self, Command, Place, Read, Run, Want};
use skein_io::kernel::Spawn;
use skein_world::Inherited;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
#[derive(Debug)]
pub struct Git {
    command: Option<Command>,
    run: Option<Run>,
}
impl Git {
    #[must_use]
    pub fn host(spawn: &Spawn, inherited: &Inherited, scenario: &Scenario<'_>, shared: Shared, seed: u64) -> Hosted {
        let command = face::check(&spawn.args, &spawn.env);
        let mut jobs = VecDeque::from([Job::Verify(spawn.program.clone())]);
        match &command {
            Command::RevParse { git_dir, .. } | Command::LsTree { git_dir, .. } | Command::CatFileBatch { git_dir } => {
                jobs.push_back(Job::Directory(git_dir.clone()));
            }
            Command::DiffNoIndex { before, after } => {
                jobs.push_back(Job::Read(before.clone()));
                jobs.push_back(Job::Read(after.clone()));
            }
        }
        let delay = scenario.pace.git(&mut super::rng(seed, spawn), 0);
        let mut hosted =
            Hosted::new(Tool::Git(Self { command: Some(command), run: None }), shared, inherited, jobs, delay, None);
        hosted.per_mib = scenario.pace.per_mib;
        hosted
    }
    pub(super) fn prepare(&mut self, files: &mut BTreeMap<Box<[u8]>, Read>, directories: &BTreeSet<Box<[u8]>>) {
        let command = self.command.take().expect("git is prepared once");
        let place = match &command {
            Command::RevParse { git_dir, .. } | Command::LsTree { git_dir, .. } | Command::CatFileBatch { git_dir } => {
                if directories.contains(git_dir) {
                    Place::Repository
                } else {
                    Place::NotARepository
                }
            }
            Command::DiffNoIndex { before, after } => Place::Files {
                before: files.remove(before).expect("read before"),
                after: files.remove(after).expect("read after"),
            },
        };
        self.run = Some(Run::new(command, place));
    }
    pub(super) fn input(&mut self, bytes: &[u8]) {
        let run = self.run.as_mut().expect("prepared git");
        if bytes.is_empty() {
            run.end();
        } else {
            run.input(bytes);
        }
    }
    pub(super) fn next(&mut self, shared: &Shared) -> Want {
        self.run.as_mut().expect("prepared git").next(&shared.borrow().repositories)
    }
}
