use crate::compose::{Present, Settings};
pub fn task_of(settings: &Settings) -> &Present {
    let mut task = None;
    for present in &settings.tasks {
        if !present.implicit {
            assert!(task.is_none(), "one named task");
            task = Some(present);
        }
    }
    task.expect("named task")
}

pub fn host_cases() -> [(&'static [u8], crate::errors::Host); 3] {
    use crate::errors::Host;
    [
        (b"absent-lab", Host::Unknown),
        (b"rhi-prod", Host::Elsewhere { environment: Box::from(b"prod".as_slice()) }),
        (b"chat-lab", Host::Container),
    ]
}

pub const TEMPLATE_ENTRIES: u32 = 5; // Four removed_task entries and one template.
pub const SHARED_TEMPLATE_ENTRIES: u32 = TEMPLATE_ENTRIES + 1;

pub fn target_with(placement: &str, values: [Option<&str>; 6]) -> (crate::revision::Documents, crate::compose::Target) {
    use super::{
        bytes,
        composition::{at, int, map, null},
    };
    use crate::revision::{self, Oid, Task};
    let (mut documents, target) = super::debian_layers([None; 6]);
    let [fleet, base, member, environment, host, host_member] = values;
    for (path, value, member) in [
        (b"fleet.json".as_slice(), fleet, false),
        (b"hosts/stoa/stoa.json", base, false),
        (b"hosts/stoa/stoa.json", member, true),
        (b"envs/lab.json", environment, false),
        (b"hosts/stoa/stoa-lab.json", host, false),
        (b"hosts/stoa/stoa-lab.json", host_member, true),
    ] {
        let index = documents.find(path).expect("layer");
        let mut document = documents.get(index).expect("document").clone();
        let layer = if member {
            super::entries::only_mut(&mut document.members.as_mut().expect("members").each)
                .layer
                .as_mut()
                .expect("member")
        } else {
            &mut document.layer
        };
        layer.tasks = match value {
            None => Box::new([]),
            Some(value) => Box::new([Task {
                name: bytes::copy_of(b"hearth"),
                at: at(1, 2),
                value: if value == "null" {
                    null(at(1, 10))
                } else if value == "settings" {
                    map(at(1, 10), &[(b"x", at(1, 11), int(at(1, 15), 42))])
                } else {
                    map(at(1, 10), &[])
                },
            }]),
        };
        documents.set(&Oid(bytes::copy_of(path)), document);
    }
    let index = target.base_env;
    let mut document = documents.get(index).expect("host").clone();
    document.layer.placement = Some(revision::Placement {
        at: at(1, 20),
        kind: if placement == "container" { revision::Runs::Container } else { revision::Runs::Metal },
        firmware: None,
        hardware: None,
        on: None,
        mutable: false,
    });
    documents.set(&Oid(bytes::copy_of(b"hosts/stoa/stoa-lab.json")), document);
    let empty = revision::Document {
        at: at(1, 1),
        members: None,
        layer: revision::Layer { debian: None, placement: None, tasks: Box::new([]) },
    };
    documents = super::with_document(&documents, b"tasks/hearth/task.json", empty);
    documents.set_declaration(&Oid(bytes::copy_of(b"tasks/hearth/task.json")), super::declaration(&[], &[]));
    (documents, target)
}

pub fn hearth_settings(documents: &crate::revision::Documents, target: &crate::compose::Target) -> Settings {
    composed_settings(target, documents, &crate::schema::Defaults::new())
}
pub fn hearth_errors(
    documents: &crate::revision::Documents,
    target: &crate::compose::Target,
) -> Box<[crate::errors::Error]> {
    let mut errors = crate::errors::Collector::new(&crate::limits::SHIPPED);
    crate::compose::rules(documents, &mut errors);
    crate::compose::reserved(target, documents, &mut errors);
    errors.finish().0
}
pub fn hearth_task_names(settings: &Settings) -> Box<[(&[u8], bool)]> {
    let mut names = skein_lib::List::with_capacity(u32::try_from(settings.tasks.len()).expect("task count"));
    for task in &settings.tasks {
        names.push((task.name.as_ref(), task.implicit)).expect("task count");
    }
    names.into_boxed()
}
pub fn hearth_fault(rule: crate::errors::Rule) -> crate::errors::Kind {
    crate::errors::Kind::Reserved(crate::errors::Reserved { rule, what: Box::from(b"hearth".as_slice()) })
}
pub fn target_without_hearth(placement: &str) -> (crate::revision::Documents, crate::compose::Target) {
    let (documents, target) = target_with(placement, [None; 6]);
    let mut entries = skein_lib::List::with_capacity(u32::try_from(documents.entries().len()).expect("document count"));
    for entry in documents.entries() {
        if entry.path.as_ref() != b"tasks/hearth/task.json" {
            entries.push(entry.clone()).expect("document count");
        }
    }
    let mut absent = crate::revision::Documents::new(entries.as_slice());
    for entry in &entries {
        let index = documents.find(&entry.path).expect("document");
        if let Some(document) = documents.get(index) {
            absent.set(&entry.oid, document.clone());
        }
    }
    (absent, target)
}

pub fn composition_rules(documents: &crate::revision::Documents) -> Box<[crate::errors::Error]> {
    let mut errors = crate::errors::Collector::new(&crate::limits::SHIPPED);
    crate::compose::rules(documents, &mut errors);
    errors.finish().0
}
pub fn composed_settings(
    target: &crate::compose::Target,
    documents: &crate::revision::Documents,
    defaults: &crate::schema::Defaults,
) -> Settings {
    crate::compose::compose(target, documents, defaults, &mut crate::compose::Merge::new(&crate::limits::SHIPPED))
}
pub fn discover(documents: &crate::revision::Documents) -> crate::compose::Discovery {
    crate::compose::targets(
        documents,
        &crate::limits::SHIPPED,
        &mut crate::errors::Collector::new(&crate::limits::SHIPPED),
    )
    .expect("targets")
}
pub fn decode_error(
    path: &[u8],
    position: Option<crate::revision::Position>,
    decode: crate::errors::Decode,
    task: Option<Box<[u8]>>,
) -> crate::errors::Error {
    crate::errors::Error {
        side: None,
        kind: crate::errors::Kind::Decode(decode),
        class: crate::errors::Class::Configuration,
        at: Some(crate::errors::Place { path: super::bytes::copy_of(path), position }),
        also: None,
        target: None,
        task,
    }
}

pub const SCHEMA_FAULTS: u32 = 16; // The schema validator's bounded diagnostics per task.
pub const DOCUMENT_BYTES: u32 = 16; // Below error_call's 24-byte documents.
pub const REVISION_BYTES: u64 = 64; // Two 24-byte documents fit, the third crosses.
pub const STATIC_BYTES: u32 = 8; // The oversized static fixture is one byte larger.
pub const SINGLE_REQUEST: u32 = 1; // The call asks one read at a time.
pub const TWO_ERRORS: u32 = 2;
pub const THREE_ERRORS: u32 = TWO_ERRORS + 1;
pub const EIGHT_ERRORS: u32 = 8; // Holds every row of the four-row sorting fixture.

pub fn composition_merge() -> crate::compose::Merge {
    crate::compose::Merge::new(&crate::limits::SHIPPED)
}
pub fn composition_collector() -> crate::errors::Collector {
    crate::errors::Collector::new(&crate::limits::SHIPPED)
}
