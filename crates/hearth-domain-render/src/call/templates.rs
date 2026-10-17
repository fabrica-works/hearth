//! One template per resume, retaining each target's validated task context.
use super::Side;
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
    vocabulary::{Answer, Class, Outcome},
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::{List, Map};
#[derive(Debug)]
pub(crate) struct Target {
    meta: Meta,
    meta_digest: Digest,
    tasks: Box<[schema::Checked]>,
    task: usize,
    entry: usize,
    roots: Option<Roots>,
    tree: Map<Box<[u8]>, TreeEntry>,
    valid: bool,
}
impl Target {
    pub fn new(meta: Meta, meta_digest: Digest, checked: schema::Validated, config: Config, capacity: u32) -> Self {
        let mut tree = Map::with_capacity(capacity);
        for file in config.tree {
            tree.insert(file.path.clone(), file).expect("tree count");
        }
        Self { meta, meta_digest, tasks: checked.tasks, task: 0, entry: 0, roots: None, tree, valid: true }
    }
    fn run(&mut self, side: &mut Side) -> bool {
        for _ in 0..=self.tasks.len() {
            let Some(task) = self.tasks.get_mut(self.task) else { return false };
            if self.roots.is_none() {
                self.roots = Some(Roots {
                    target: context::target(&self.meta),
                    debian: Value::Map(Box::new([])),
                    settings: core::mem::replace(&mut task.settings, Value::Map(Box::new([]))),
                    tasks: core::mem::replace(&mut task.tasks, Value::Map(Box::new([]))),
                    exports: Value::Map(Box::new([])),
                });
            }
            for entry in side.entries.get(self.entry..).expect("listing cursor") {
                self.entry = self.entry.saturating_add(1);
                let revision::Class::Tree { task: name, path } = revision::class(&entry.path) else { continue };
                if name != task.name.as_ref() || !path.ends_with(b".j2") {
                    continue;
                }
                let Some(template) = side.templates.get(&entry.oid) else { continue };
                match template::run(
                    template,
                    &self.roots.as_ref().expect("task roots").context(),
                    &side.limits,
                    u64::from(side.limits.rendered_file),
                ) {
                    Ok(bytes) => {
                        let digest = Digest(skein_lib::sha256::digest(&bytes));
                        side.sizes
                            .insert(digest, u64::try_from(bytes.len()).expect("file bound"))
                            .expect("listing capacity");
                        let file = parts::config::rendered(path, entry.mode, digest);
                        self.tree.insert(file.path.clone(), file).expect("listing capacity");
                    }
                    Err(fault) => {
                        self.valid = false;
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
        if !target.valid {
            self.outcome = self.outcome.worst(Outcome::Failed);
            return None;
        }
        let mut files = List::with_capacity(target.tree.len());
        for (_, file) in &target.tree {
            files.push(file.clone()).expect("tree count");
        }
        let config = Config { tree: files.into_boxed() };
        let parts = Box::new([parts::Part {
            digest: parts::config::digest(&config),
            record: parts::PartRecord::Config(config),
        }]);
        Some(Answer::Spec(parts::Spec {
            digest: parts::spec_digest(&target.meta_digest, parts.as_ref()),
            meta: target.meta,
            meta_digest: target.meta_digest,
            commit: self.commit.clone().expect("listed commit"),
            parts,
        }))
    }
}
