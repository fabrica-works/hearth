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
    pub settings: Box<[compose::Settings]>,
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
    let mut settings = List::with_capacity(u32::try_from(targets.len()).expect("targets"));
    for target in &targets {
        settings.push(compose::compose(target, &documents, &defaults, &mut merge)).expect("targets");
    }
    let settings = settings.into_boxed();
    Readers { documents, defaults, targets, settings, faults, limits: SHIPPED }
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
    pub fn prime(&self, cache: &mut Cache, name: &[u8]) -> usize {
        let revision = schema::Revision {
            documents: &self.documents,
            defaults: &self.defaults,
            listing: self.documents.entries(),
        };
        let fleet = Fleet {
            revision: &revision,
            targets: &self.targets,
            settings: &self.settings,
            faults: &self.faults,
            limits: &self.limits,
        };
        for pairs in 0..=self.targets.len().saturating_mul(2) {
            if !cache.prime(self.target(name), &fleet) {
                return pairs;
            }
        }
        panic!("bounded pairs")
    }
    pub fn root(&self, cache: &Cache, name: &[u8]) -> Result<crate::evaluate::Value, Box<errors::Error>> {
        let revision = schema::Revision {
            documents: &self.documents,
            defaults: &self.defaults,
            listing: self.documents.entries(),
        };
        let fleet = Fleet {
            revision: &revision,
            targets: &self.targets,
            settings: &self.settings,
            faults: &self.faults,
            limits: &self.limits,
        };
        let (_, Some(declaration)) = self.documents.declaration(b"reader").expect("reader") else {
            panic!("declaration")
        };
        cache.root(self.target(name), declaration, &fleet)
    }
    pub fn setting_fault(&mut self) {
        for (settings, target) in self.settings.iter_mut().zip(&self.targets) {
            if target.name.as_ref() != b"ion-prod" {
                continue;
            }
            for task in &mut settings.tasks {
                if task.name.as_ref() == b"vpn" {
                    task.settings = super::schemas::composed(&map(
                        at(1, 9),
                        &[(b"x", at(1, 10), revision::Value { at: at(1, 15), kind: revision::Kind::Bool(false) })],
                    ));
                }
            }
        }
    }
    pub fn cache(&self, name: &[u8]) -> Cache {
        let mut cache = Cache::new(&self.limits);
        self.prime(&mut cache, name);
        cache
    }
    pub fn remove_exporters(&mut self) {
        for settings in &mut self.settings {
            let mut tasks = List::with_capacity(u32::try_from(settings.tasks.len()).expect("tasks"));
            for task in &settings.tasks {
                if task.name.as_ref() != b"vpn" {
                    tasks.push(task.clone()).expect("tasks");
                }
            }
            settings.tasks = tasks.into_boxed();
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
