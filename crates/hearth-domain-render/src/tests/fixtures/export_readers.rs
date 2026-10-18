use super::{
    composition::{at, int, map},
    containers, declaration, declared_spec, read, with_document,
};
use crate::{
    compose, errors,
    evaluate::Missing,
    exports::{Cache, Fleet},
    limits::{Limits, SHIPPED},
    revision::{self, declaration::Form},
    schema,
};
use skein_lib::{List, bytes::copy_of};
pub const SMALL_EXPORTED: Limits = Limits { exported: 64, ..SHIPPED };
pub struct Readers {
    pub documents: revision::Documents,
    pub defaults: schema::Defaults,
    pub targets: Box<[compose::Target]>,
    pub tasks: Box<[Box<[compose::Index]>]>,
    pub faults: errors::Collector,
    pub limits: Limits,
}
pub fn readers(own: bool) -> Readers {
    let mut vpn = super::declared(
        Form::Int,
        super::parts([super::entry_output([
            super::name(0, b"settings", Missing::Error),
            super::key(9, b"x", Missing::Error),
        ])]),
        false,
    );
    vpn.settings = Box::new([declared_spec(b"x")]);
    let mut reading = read(b"vpn", &[]);
    reading.exports = true;
    let reader = declaration(&[], &[reading]);
    let mut documents = super::declarations(&[(b"reader", Some(reader)), (b"vpn", Some(vpn))]);
    for path in [b"envs/prod.json".as_slice(), b"envs/lab.json"] {
        documents = with_document(&documents, path, containers::document(None));
    }
    for (path, exporter) in [
        (b"hosts/carbon/carbon-lab.json".as_slice(), own),
        (b"hosts/carbon/carbon-prod.json", own),
        (b"hosts/ion/ion-lab.json", true),
        (b"hosts/ion/ion-prod.json", true),
    ] {
        let mut document = containers::document(Some(containers::placement(None)));
        let mut tasks = List::with_capacity(2);
        tasks
            .push(revision::Task { name: copy_of(b"reader"), at: at(1, 2), value: map(at(1, 9), &[]) })
            .expect("two tasks");
        if exporter {
            tasks
                .push(revision::Task {
                    name: copy_of(b"vpn"),
                    at: at(1, 2),
                    value: map(at(1, 9), &[(b"x", at(1, 10), int(at(1, 15), 7))]),
                })
                .expect("two tasks");
        }
        document.layer.tasks = tasks.into_boxed();
        documents = with_document(&documents, path, document);
    }
    built(documents)
}
pub fn built(documents: revision::Documents) -> Readers {
    let mut faults = errors::Collector::new(&SHIPPED);
    let targets = compose::targets(&documents, &SHIPPED, &mut faults).expect("targets").targets;
    let mut merge = compose::Merge::new(&SHIPPED);
    let mut schemas = List::with_capacity(16);
    let defaults = schema::declarations(
        &documents,
        documents.entries(),
        &mut schema::Validator::new(&SHIPPED),
        &mut merge,
        &mut schemas,
    );
    assert_eq!(schemas.as_slice(), [], "sound declarations");
    let tasks = super::target_tasks(&targets, &documents);
    Readers { documents, defaults, targets, tasks, faults, limits: SHIPPED }
}
impl Readers {
    pub fn target(&self, name: &[u8]) -> &compose::Target {
        for target in &self.targets {
            if target.name.as_ref() == name {
                return target;
            }
        }
        panic!("fixture target")
    }
    fn fleet(&self) -> Fleet<'_> {
        Fleet {
            revision: schema::Revision {
                documents: &self.documents,
                defaults: &self.defaults,
                listing: self.documents.entries(),
            },
            targets: &self.targets,
            tasks: &self.tasks,
            faults: &self.faults,
            limits: &self.limits,
        }
    }
    pub fn prime(&self, cache: &mut Cache, name: &[u8]) -> usize {
        for pairs in 0..=self.targets.len().saturating_mul(2) {
            if !cache.prime(self.target(name), &self.fleet()) {
                return pairs;
            }
        }
        panic!("bounded pairs")
    }
    pub fn root(&self, cache: &Cache, name: &[u8]) -> Result<crate::evaluate::Value, Box<errors::Error>> {
        let (_, Some(declaration)) = self.documents.declaration(b"reader").expect("reader") else {
            panic!("declaration")
        };
        cache.root(self.target(name), declaration, &self.fleet())
    }
    pub fn setting_fault(&mut self) {
        let path = b"hosts/ion/ion-prod.json";
        let index = self.documents.find(path).expect("exporter document");
        let mut document = self.documents.get(index).expect("document").clone();
        for task in &mut document.layer.tasks {
            if task.name.as_ref() == b"vpn" {
                task.value = map(
                    at(1, 9),
                    &[(b"x", at(1, 10), revision::Value { at: at(1, 15), kind: revision::Kind::Bool(false) })],
                );
            }
        }
        self.documents = with_document(&self.documents, path, document);
    }
    pub fn cache(&self, name: &[u8]) -> Cache {
        let mut cache = Cache::new(&self.limits);
        self.prime(&mut cache, name);
        cache
    }
    pub fn remove_exporters(&mut self) {
        for indexes in &mut self.tasks {
            let mut kept = List::with_capacity(u32::try_from(indexes.len()).expect("task count"));
            for index in &**indexes {
                if index.name(&self.documents) != b"vpn" {
                    kept.push(*index).expect("task count");
                }
            }
            *indexes = kept.into_boxed();
        }
    }
    pub fn refused(&mut self, path: &[u8]) {
        self.faults.push(errors::refused(path, Some(at(2, 3)), errors::Decode::Unexpected, None));
    }
}

pub fn keys(entries: &[crate::evaluate::Entry]) -> Box<[&[u8]]> {
    let mut keys = List::with_capacity(u32::try_from(entries.len()).expect("entries"));
    for entry in entries {
        keys.push(entry.key.as_ref()).expect("entries");
    }
    keys.into_boxed()
}
