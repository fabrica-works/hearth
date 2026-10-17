//! One template per resume, retaining each target's validated task context.
use super::{Side, files};
use crate::kinds::files::{Make, Plan};
use crate::{
    errors::{Error, Kind, Place},
    evaluate::{
        Value,
        context::{self, Roots},
        template,
    },
    parts::{self, config::TreeEntry, meta::Meta},
    revision, schema,
    vocabulary::{Answer, Class, Outcome, Output, Request},
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::{List, Map, Queue, Token};
pub(crate) struct Prepared {
    pub accounts: crate::kinds::users::Accounts,
    pub installation: super::scripts::Installation,
    pub expansion: crate::entries::Expansion,
    pub secrets: parts::secrets::Secrets,
    pub packages: parts::packages::Packages,
    pub live: parts::live::Live,
    pub boot: parts::boot::Boot,
    pub exports: parts::exports::Exports,
    pub export_roots: Value,
}
#[derive(Debug)]
pub(crate) struct Target {
    meta: Meta,
    containers: Box<[crate::compose::Container]>,
    meta_digest: Digest,
    exports: parts::exports::Exports,
    export_roots: Value,
    tasks: Box<[schema::Checked]>,
    debian: Value,
    pub(crate) expansion: crate::entries::Expansion,
    secrets: parts::secrets::Secrets,
    packages: parts::packages::Packages,
    live: parts::live::Live,
    boot: parts::boot::Boot,
    task: usize,
    rendered: u64,
    entry: usize,
    roots: Option<Roots>,
    tree: Map<Box<[u8]>, TreeEntry>,
    accounts: crate::kinds::users::Accounts,
    plans: Box<[Plan]>,
    outcome: Outcome,
    keeping: Option<Keeping>,
    presets: Box<[parts::config::Preset]>,
    tmpfiles: Box<[parts::config::Tmpfile]>,
    scripts: Box<[crate::kinds::scripts::Source]>,
    script_rows: Map<(u16, Box<[u8]>), parts::config::Script>,
    script_done: bool,
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
        let Prepared { accounts, installation, expansion, secrets, packages, live, boot, exports, export_roots } =
            prepared;
        let super::scripts::Installation { plans, presets, tmpfiles, scripts } = installation;
        let script_rows = Map::with_capacity(u32::try_from(scripts.len()).expect("task bound"));
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
                    Make::Rendered { .. } | Make::Copied { .. } => continue,
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
            exports,
            export_roots,
            tasks: checked.tasks,
            debian,
            expansion,
            secrets,
            packages,
            live,
            boot,
            task: 0,
            rendered: 0,
            entry: 0,
            roots: None,
            tree,
            accounts,
            plans,
            outcome: Outcome::Done,
            keeping: None,
            presets,
            tmpfiles,
            scripts,
            script_rows,
            script_done: false,
            request: None,
        }
    }
    fn run(&mut self, side: &mut Side) -> bool {
        if let Some(request) = side.copies(&self.plans) {
            self.request = Some(request);
            return true;
        }
        for _ in 0..=self.tasks.len() {
            let Some(task) = self.tasks.get_mut(self.task) else { return false };
            if self.roots.is_none() {
                self.roots = Some(Roots {
                    target: context::target(&self.meta, &self.containers),
                    debian: self.debian.clone(),
                    settings: core::mem::replace(&mut task.settings, Value::Map(Box::new([]))),
                    tasks: core::mem::replace(&mut task.tasks, Value::Map(Box::new([]))),
                    exports: crate::exports::of(&self.export_roots, &task.name).clone(),
                });
            }
            let task_name = task.name.clone();
            let count = self.plans.get(self.task).expect("task plan").installs.len();
            for index in self.entry..count {
                let install =
                    self.plans.get(self.task).expect("task plan").installs.get(index).expect("plan cursor").clone();
                self.entry = self.entry.saturating_add(1);
                let (path, via, mode) = match &install.make {
                    Make::Rendered { template, via, mode } => (template, via, *mode),
                    Make::Copied { source, mode } => {
                        self.tree
                            .insert(install.path.clone(), copied(side, &install, source, *mode))
                            .expect("plan capacity");
                        continue;
                    }
                    Make::Static { .. } | Make::Link { .. } | Make::Directory { .. } => continue,
                };
                if let Some((digest, bytes)) = self.render(side, path, &task_name, via.as_ref()) {
                    self.keeping = Some(Keeping::File(TreeEntry {
                        path: install.path.clone(),
                        node: parts::config::Node::File {
                            content_digest: digest,
                            mode,
                            owner: install.owner.clone(),
                            group: install.group.clone(),
                        },
                    }));
                    self.request =
                        Some(Request::Keep { request: Token::new(4), path: path.clone(), sha256: digest, bytes });
                    side.ready = false;
                }
                return true;
            }
            if !self.script_done {
                self.script_done = true;
                if self.script(side, &task_name) {
                    return true;
                }
            }
            self.task = self.task.saturating_add(1);
            self.script_done = false;
            self.entry = 0;
            self.roots = None;
        }
        false
    }
}
impl Side {
    pub(super) fn prepare_rendering(
        &mut self,
        meta: Meta,
        containers: Box<[crate::compose::Container]>,
        meta_digest: Digest,
        checked: schema::Validated,
        debian: Value,
        prepared: Prepared,
    ) {
        let capacity = self.plan_capacity(&prepared.installation.plans);
        let mut sizes = Map::with_capacity(
            self.sizes
                .capacity()
                .saturating_add(u32::try_from(prepared.installation.scripts.len()).expect("task bound")),
        );
        for (digest, size) in &self.sizes {
            sizes.insert(*digest, *size).expect("size bound");
        }
        self.sizes = sizes;
        self.rendering = Some(Target::new(meta, containers, meta_digest, checked, debian, prepared, capacity));
    }

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
        let keeping = target.keeping.take().expect("one rendered file waiting");
        if let Some(digest) = digest {
            match keeping {
                Keeping::File(file) => {
                    let parts::config::Node::File { content_digest, .. } = &file.node else {
                        unreachable!("rendered file")
                    };
                    assert_eq!(digest, *content_digest, "kept rendered digest");
                    target.tree.insert(file.path.clone(), file).expect("listing capacity");
                }
                Keeping::Script(script) => {
                    assert_eq!(digest, script.content_digest, "kept rendered digest");
                    target.script_rows.insert((script.order, script.task.clone()), script).expect("task bound");
                }
            }
        } else {
            target.outcome = target.outcome.worst(Outcome::TryAgain);
            self.outcome = self.outcome.worst(Outcome::TryAgain);
        }
        self.ready = true;
    }
    pub(super) fn plan_capacity(&mut self, plans: &[Plan]) -> u32 {
        let tree_capacity = files::capacity(plans);
        let mut capacity = self.sizes.capacity();
        for plan in plans {
            for install in &plan.installs {
                if let Make::Rendered { .. } = install.make {
                    capacity = capacity.saturating_add(1);
                }
            }
        }
        if capacity == self.sizes.capacity() {
            return tree_capacity;
        }
        let mut sizes = Map::with_capacity(capacity);
        for (digest, size) in &self.sizes {
            sizes.insert(*digest, *size).expect("rendered count");
        }
        self.sizes = sizes;
        tree_capacity
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
        let mut config = parts::config::account_rows(&target.accounts, files.into_boxed());
        config.presets = target.presets;
        config.tmpfiles = target.tmpfiles;
        let mut scripts = List::with_capacity(target.script_rows.len());
        for (_, row) in &target.script_rows {
            scripts.push(row.clone()).expect("task bound");
        }
        config.scripts = scripts.into_boxed();
        let parts = Box::new([
            parts::Part { digest: parts::boot::digest(&target.boot), record: parts::PartRecord::Boot(target.boot) },
            parts::Part {
                digest: parts::packages::digest(&target.packages),
                record: parts::PartRecord::Packages(target.packages),
            },
            parts::Part { digest: parts::config::digest(&config), record: parts::PartRecord::Config(config) },
            parts::Part {
                digest: parts::secrets::digest(&target.secrets),
                record: parts::PartRecord::Secrets(target.secrets),
            },
            parts::Part { digest: parts::live::digest(&target.live), record: parts::PartRecord::Live(target.live) },
            parts::Part {
                digest: parts::exports::digest(&target.exports),
                record: parts::PartRecord::Exports(target.exports),
            },
        ]);
        drop(target.expansion);
        Some(Answer::Spec(parts::Spec {
            resolution: None,
            digest: parts::spec_digest(&target.meta_digest, parts.as_ref()),
            meta: target.meta,
            meta_digest: target.meta_digest,
            commit: self.commit.clone().expect("listed commit"),
            parts,
        }))
    }
}

fn element<'c>(
    expansion: &crate::entries::Expansion,
    documents: &revision::Documents,
    task: &[u8],
    via: Option<&crate::kinds::files::Via>,
    context: &crate::evaluate::Context<'c>,
    limits: &crate::limits::Limits,
) -> Option<(crate::evaluate::Operand<'c>, crate::evaluate::Operand<'c>)> {
    let via = via?;
    let index = via.element?;
    let mut at = None;
    for expanded in &expansion.tasks {
        if expanded.task.as_ref() != task {
            continue;
        }
        for group in &expanded.groups {
            if group.map == revision::declaration::Map::Files {
                at = Some(
                    group.entries.get(usize::try_from(via.entry).expect("entry index")).expect("retained entry").at,
                );
            }
        }
    }
    let (_, Some(declaration)) = documents.declaration(task).expect("task declaration") else {
        unreachable!("validated declaration")
    };
    let mut declared = None;
    for group in &declaration.entries {
        if group.map != revision::declaration::Map::Files {
            continue;
        }
        for entry in &group.entries {
            if Some(entry.key.at()) == at {
                declared = Some(entry);
            }
        }
    }
    let entry = declared.expect("retained declaration");
    Some(crate::entries::element(entry, index, context, limits).expect("retained element"))
}

fn copied(side: &Side, install: &crate::kinds::files::Install, source: &[u8], mode: u16) -> TreeEntry {
    let entry = side.listed_entry(source).expect("checked copy");
    TreeEntry {
        path: install.path.clone(),
        node: parts::config::Node::File {
            content_digest: *side.cached.get(&entry.oid).expect("copy read before rendering"),
            mode,
            owner: install.owner.clone(),
            group: install.group.clone(),
        },
    }
}

#[derive(Debug)]
enum Keeping {
    File(TreeEntry),
    Script(parts::config::Script),
}
mod scripts;
