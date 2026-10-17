//! One template per resume, retaining each target's validated task context.
use super::Side;
use crate::kinds::files::{Make, Plan};
use crate::{
    errors::{Error, Kind, Place},
    evaluate::{
        Value,
        context::{self, Roots},
        template,
    },
    parts::{
        self,
        config::{Config, TreeEntry},
        meta::Meta,
    },
    revision, schema,
    vocabulary::{Answer, Class, Outcome, Output, Request},
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::{List, Map, Queue, Token};
pub(crate) struct Prepared {
    pub plans: Box<[Plan]>,
    pub expansion: crate::entries::Expansion,
    pub secrets: parts::secrets::Secrets,
    pub packages: parts::packages::Packages,
}
#[derive(Debug)]
pub(crate) struct Target {
    meta: Meta,
    containers: Box<[crate::compose::Container]>,
    meta_digest: Digest,
    tasks: Box<[schema::Checked]>,
    debian: Value,
    pub(crate) expansion: crate::entries::Expansion,
    secrets: parts::secrets::Secrets,
    packages: parts::packages::Packages,
    task: usize,
    rendered: u64,
    entry: usize,
    roots: Option<Roots>,
    tree: Map<Box<[u8]>, TreeEntry>,
    plans: Box<[Plan]>,
    outcome: Outcome,
    keeping: Option<TreeEntry>,
    request: Option<Request>,
}
impl Target {
    pub fn new(
        meta: Meta,
        containers: Box<[crate::compose::Container]>,
        meta_digest: Digest,
        checked: schema::Validated,
        debian: Value,
        prepared: Prepared,
        capacity: u32,
    ) -> Self {
        let Prepared { plans, expansion, secrets, packages } = prepared;
        let mut tree = Map::with_capacity(capacity);
        for plan in &plans {
            for install in &plan.installs {
                let node = match &install.make {
                    Make::Static { sha256, mode } => parts::config::Node::File {
                        content_digest: *sha256,
                        mode: *mode,
                        owner: install.owner.clone(),
                        group: install.group.clone(),
                    },
                    Make::Link { target } => parts::config::Node::Link {
                        target: target.clone(),
                        owner: install.owner.clone(),
                        group: install.group.clone(),
                    },
                    Make::Directory { mode } => parts::config::Node::Directory {
                        mode: *mode,
                        owner: install.owner.clone(),
                        group: install.group.clone(),
                    },
                    Make::Rendered { .. } => continue,
                };
                if !tree.contains_key(&install.path) {
                    tree.insert(install.path.clone(), TreeEntry { path: install.path.clone(), node })
                        .expect("plan capacity");
                } // transitional: hearth 22
            }
        }
        Self {
            meta,
            containers,
            meta_digest,
            tasks: checked.tasks,
            debian,
            expansion,
            secrets,
            packages,
            task: 0,
            rendered: 0,
            entry: 0,
            roots: None,
            tree,
            plans,
            outcome: Outcome::Done,
            keeping: None,
            request: None,
        }
    }
    fn run(&mut self, side: &mut Side) -> bool {
        for _ in 0..=self.tasks.len() {
            let Some(task) = self.tasks.get_mut(self.task) else { return false };
            if self.roots.is_none() {
                self.roots = Some(Roots {
                    target: context::target(&self.meta, &self.containers),
                    debian: self.debian.clone(),
                    settings: core::mem::replace(&mut task.settings, Value::Map(Box::new([]))),
                    tasks: core::mem::replace(&mut task.tasks, Value::Map(Box::new([]))),
                    exports: Value::Map(Box::new([])),
                });
            }
            let plan = self.plans.get(self.task).expect("task plan");
            for install in plan.installs.get(self.entry..).expect("plan cursor") {
                self.entry = self.entry.saturating_add(1);
                let (path, mode) = match &install.make {
                    Make::Rendered { template, mode } => (template, *mode),
                    Make::Static { .. } | Make::Link { .. } | Make::Directory { .. } => continue,
                };
                let mut listed = None;
                for entry in &side.entries {
                    if &entry.path == path {
                        listed = Some(entry);
                        break;
                    }
                }
                let entry = listed.expect("planned template listed");
                let Some(template) = side.templates.get(&entry.oid) else { continue };
                match template::run(
                    template,
                    &self.roots.as_ref().expect("task roots").context(),
                    &side.limits,
                    side.limits.rendered_target.saturating_sub(self.rendered),
                ) {
                    Ok(bytes) => {
                        self.rendered = self
                            .rendered
                            .checked_add(u64::try_from(bytes.len()).expect("file bound"))
                            .expect("target bound");
                        let digest = Digest(skein_lib::sha256::digest(&bytes));
                        side.sizes
                            .insert(digest, u64::try_from(bytes.len()).expect("file bound"))
                            .expect("listing capacity");
                        let file = TreeEntry {
                            path: install.path.clone(),
                            node: parts::config::Node::File {
                                content_digest: digest,
                                mode,
                                owner: install.owner.clone(),
                                group: install.group.clone(),
                            },
                        };
                        self.keeping = Some(file);
                        self.request = Some(Request::Keep {
                            request: Token::new(4),
                            path: entry.path.clone(),
                            sha256: digest,
                            bytes,
                        });
                        side.ready = false;
                    }
                    Err(fault) => {
                        self.outcome = self.outcome.worst(Outcome::Failed);
                        side.errors.push(Error {
                            side: None,
                            kind: Kind::Evaluation(fault.kind),
                            class: Class::Configuration,
                            at: Some(Place {
                                path: entry.path.clone(),
                                position: Some(template::place(&template.lines, fault.at)),
                            }),
                            also: None,
                            target: Some(self.meta.target.clone()),
                            task: Some(task.name.clone()),
                        });
                    }
                }
                return true;
            }
            self.task = self.task.saturating_add(1);
            self.entry = 0;
            self.roots = None;
        }
        false
    }
}
impl Side {
    pub(super) fn answer_for(&mut self, out: &mut Queue<Output>) -> Option<Answer> {
        let answer = self.answer();
        if answer.is_none() {
            self.template_request(out);
        }
        answer
    }
    pub(crate) fn template_request(&mut self, out: &mut Queue<Output>) {
        if let Some(target) = &mut self.rendering
            && let Some(request) = target.request.take()
        {
            out.push(Output::Request(request));
        }
    }
    pub(crate) fn kept(&mut self, digest: Option<Digest>) {
        let target = self.rendering.as_mut().expect("rendered target");
        let file = target.keeping.take().expect("one rendered file waiting");
        if let Some(digest) = digest {
            let parts::config::Node::File { content_digest, .. } = &file.node else { unreachable!("rendered file") };
            assert_eq!(digest, *content_digest, "kept rendered digest");
            target.tree.insert(file.path.clone(), file).expect("listing capacity");
        } else {
            target.outcome = target.outcome.worst(Outcome::TryAgain);
            self.outcome = self.outcome.worst(Outcome::TryAgain);
        }
        self.ready = true;
    }
    pub(super) fn size_capacity(&self) -> u32 {
        let mut capacity = u32::try_from(self.entries.len()).expect("listing count");
        for settings in &self.settings {
            for task in &settings.tasks {
                for entry in &self.entries {
                    match revision::class(&entry.path) {
                        revision::Class::Tree { task: name, path }
                            if name == task.name.as_ref() && path.ends_with(b".j2") =>
                        {
                            capacity = capacity.saturating_add(1);
                        }
                        revision::Class::Fleet
                        | revision::Class::Environment { .. }
                        | revision::Class::Host { .. }
                        | revision::Class::Declaration { .. }
                        | revision::Class::Tree { .. }
                        | revision::Class::Template { .. }
                        | revision::Class::Script { .. }
                        | revision::Class::Readme { .. }
                        | revision::Class::Refused { .. }
                        | revision::Class::Other => {}
                    }
                }
            }
        }
        capacity
    }
    pub(super) fn template(&mut self) -> Option<Answer> {
        let mut target = self.rendering.take().expect("target prepared");
        if target.run(self) {
            self.rendering = Some(target);
            return None;
        }
        if target.outcome != Outcome::Done {
            self.outcome = self.outcome.worst(target.outcome);
            return None;
        }
        let mut files = List::with_capacity(target.tree.len());
        for (_, file) in &target.tree {
            files.push(file.clone()).expect("tree count");
        }
        let config = Config { tree: files.into_boxed() };
        let parts = Box::new([
            parts::Part {
                digest: parts::packages::digest(&target.packages),
                record: parts::PartRecord::Packages(target.packages),
            },
            parts::Part { digest: parts::config::digest(&config), record: parts::PartRecord::Config(config) },
            parts::Part {
                digest: parts::secrets::digest(&target.secrets),
                record: parts::PartRecord::Secrets(target.secrets),
            },
        ]);
        drop(target.expansion);
        Some(Answer::Spec(parts::Spec {
            digest: parts::spec_digest(&target.meta_digest, parts.as_ref()),
            meta: target.meta,
            meta_digest: target.meta_digest,
            commit: self.commit.clone().expect("listed commit"),
            parts,
        }))
    }
}
