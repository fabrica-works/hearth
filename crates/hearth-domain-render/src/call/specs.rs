//! Count held specs as records of values (domain/render.md, 2.3).
mod config;
mod packages;
mod values;
use crate::{evaluate::ELEMENT, parts, resolution};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::Map;
pub(super) fn bytes(value: &[u8]) -> u64 {
    u64::try_from(value.len()).expect("held bytes fit u64")
}
fn optional(value: Option<&[u8]>) -> u64 {
    match value {
        Some(value) => bytes(value),
        None => 0,
    }
}
pub(super) fn record(fields: &[(&[u8], u64)]) -> u64 {
    let mut size = 0_u64;
    for (key, value) in fields {
        size = size.saturating_add(ELEMENT).saturating_add(bytes(key)).saturating_add(*value);
    }
    size
}
fn strings(values: &[Box<[u8]>]) -> u64 {
    let mut held = 0_u64;
    for value in values {
        held = held.saturating_add(ELEMENT).saturating_add(bytes(value));
    }
    held
}
pub(super) fn spec(spec: &parts::Spec, max: u64) -> u64 {
    let meta = &spec.meta;
    let p = &meta.placement;
    let kind: &[u8] = match p.kind {
        crate::revision::Runs::Metal => b"metal",
        crate::revision::Runs::Vm => b"vm",
        crate::revision::Runs::Container => b"container",
    };
    let firmware: &[u8] = match p.firmware {
        Some(crate::revision::Firmware::Efi) => b"efi",
        Some(crate::revision::Firmware::Bios) => b"bios",
        None => b"",
    };
    let placement = record(&[
        (b"kind", bytes(kind)),
        (b"firmware", bytes(firmware)),
        (b"hardware", optional(p.hardware.as_deref())),
        (b"on", optional(p.on.as_deref())),
    ]);
    let meta = record(&[
        (b"target", bytes(&meta.target)),
        (b"base", bytes(&meta.base)),
        (b"member", optional(meta.member.as_deref())),
        (b"environment", bytes(&meta.environment)),
        (b"placement", placement),
    ]);
    let resolution = match spec.resolution.as_deref() {
        Some(crate::keys::Resolution::Needed { .. }) => record(&[(b"key", 32)]),
        Some(crate::keys::Resolution::Known { .. }) => {
            record(&[(b"key", 32), (b"keys", record(&[(b"packages", 32), (b"software", 32), (b"config", 32)]))])
        }
        None => 0,
    };
    let mut held = record(&[
        (b"meta", meta),
        (b"meta_digest", 32),
        (b"commit", bytes(&spec.commit.0)),
        (b"digest", 32),
        (b"parts", 0),
        (b"resolution", resolution),
    ]);
    for part in &spec.parts {
        held = held.saturating_add(ELEMENT).saturating_add(self::part(part, max));
    }
    held
}
fn part(part: &parts::Part, max: u64) -> u64 {
    let size = match &part.record {
        parts::PartRecord::Config(value) => config::config(value),
        parts::PartRecord::Packages(value) => packages::packages(value),
        parts::PartRecord::Boot(value) => record(&[(b"params", strings(&value.params)), (b"mutable", 0)]),
        parts::PartRecord::Data(value) => {
            let mut held = record(&[(b"entries", 0)]);
            for entry in &value.entries {
                held = held.saturating_add(ELEMENT).saturating_add(record(&[
                    (b"name", bytes(&entry.name)),
                    (
                        b"value",
                        record(&[(b"class", bytes(entry.value.class.name())), (b"path", bytes(&entry.value.path))]),
                    ),
                ]));
            }
            held
        }
        parts::PartRecord::Live(value) => {
            let mut held = record(&[(b"paths", 0)]);
            for path in &value.paths {
                held = held
                    .saturating_add(ELEMENT)
                    .saturating_add(record(&[(b"path", bytes(&path.path)), (b"consumers", 0)]));
                for consumer in &path.consumers {
                    held = held.saturating_add(ELEMENT).saturating_add(record(&[
                        (b"task", bytes(&consumer.task)),
                        (b"unit", optional(consumer.unit.as_deref())),
                        (b"action", bytes(consumer.action.name())),
                    ]));
                }
            }
            held
        }
        parts::PartRecord::Software(value) => software(value),
        parts::PartRecord::Secrets(value) => secrets(value),
        parts::PartRecord::Exports(value) => {
            let mut held = record(&[(b"tasks", 0)]);
            for task in &value.tasks {
                held = held.saturating_add(ELEMENT).saturating_add(record(&[
                    (b"task", bytes(&task.task)),
                    (b"values", values::entries(&task.values, max)),
                ]));
            }
            held
        }
    };
    record(&[(b"digest", 32), (b"record", size)])
}
fn software(value: &parts::software::Software) -> u64 {
    let mut held = record(&[(b"downloads", 0), (b"scripts", config::scripts(&value.scripts)), (b"fetch", 0)]);
    for download in &value.downloads {
        let how = match download.how {
            parts::software::How::File { .. } => record(&[(b"mode", 0)]),
            parts::software::How::Tar { .. } => record(&[(b"strip", 0)]),
        };
        held = held.saturating_add(ELEMENT).saturating_add(record(&[
            (b"path", bytes(&download.path)),
            (b"sha256", 32),
            (b"owner", bytes(&download.owner)),
            (b"group", bytes(&download.group)),
            (b"how", how),
        ]));
    }
    for fetch in &value.fetch {
        held = held.saturating_add(ELEMENT).saturating_add(record(&[(b"sha256", 32), (b"url", bytes(&fetch.url))]));
    }
    held
}
fn secrets(value: &parts::secrets::Secrets) -> u64 {
    let mut held = record(&[(b"files", 0), (b"passwords", 0), (b"identity", 0)]);
    for file in &value.files {
        held = held.saturating_add(ELEMENT).saturating_add(record(&[
            (b"name", bytes(&file.name)),
            (b"path", bytes(&file.path)),
            (b"mode", 0),
            (b"owner", bytes(&file.owner)),
            (b"group", bytes(&file.group)),
        ]));
    }
    for password in &value.passwords {
        held = held
            .saturating_add(ELEMENT)
            .saturating_add(record(&[(b"user", bytes(&password.user)), (b"secret", bytes(&password.secret))]));
    }
    for identity in &value.identity {
        held = held.saturating_add(ELEMENT).saturating_add(record(&[
            (b"name", bytes(&identity.name)),
            (b"kind", 0),
            (b"path", bytes(&identity.path)),
            (b"public_path", optional(identity.public_path.as_deref())),
            (b"mode", 0),
            (b"owner", bytes(&identity.owner)),
            (b"group", bytes(&identity.group)),
        ]));
    }
    held
}
pub(super) fn closure(packages: &[resolution::Package]) -> u64 {
    let mut held = 0_u64;
    for package in packages {
        held = held.saturating_add(ELEMENT).saturating_add(record(&[
            (b"name", bytes(&package.name)),
            (b"architecture", bytes(&package.architecture)),
            (b"version", bytes(&package.version)),
            (b"source", 0),
            (b"filename", bytes(&package.filename)),
            (b"size", 0),
            (b"sha256", 32),
        ]));
    }
    held
}
pub(super) fn files(spec: &parts::Spec, sizes: &super::Sizes, held: &mut Map<Digest, u64>) {
    for part in &spec.parts {
        match &part.record {
            parts::PartRecord::Config(config) => {
                for entry in &config.tree {
                    match &entry.node {
                        parts::config::Node::File { content_digest, .. } => file(content_digest, sizes, held),
                        parts::config::Node::Link { .. } | parts::config::Node::Directory { .. } => {}
                    }
                }
                script_files(&config.scripts, sizes, held);
            }
            parts::PartRecord::Software(software) => {
                script_files(&software.scripts, sizes, held);
                for download in &software.downloads {
                    file(&download.sha256, sizes, held);
                }
            }
            parts::PartRecord::Packages(packages) => {
                if let Some(digest) = packages.initramfs_file {
                    file(&digest, sizes, held);
                }
            }
            parts::PartRecord::Data(_)
            | parts::PartRecord::Boot(_)
            | parts::PartRecord::Secrets(_)
            | parts::PartRecord::Exports(_)
            | parts::PartRecord::Live(_) => {}
        }
    }
}
fn script_files(scripts: &[parts::config::Script], sizes: &super::Sizes, held: &mut Map<Digest, u64>) {
    for script in scripts {
        file(&script.content_digest, sizes, held);
    }
}
fn file(digest: &Digest, sizes: &super::Sizes, held: &mut Map<Digest, u64>) {
    if let Some(size) = sizes.get(digest) {
        held.insert(*digest, *size).expect("file size bound");
    }
}

pub(super) fn held(
    made: &Map<Box<[u8]>, crate::diff::Made>,
    sizes: &super::Sizes,
    finding: &super::resolution::Finding,
    max: u64,
) -> u64 {
    let mut total = finding.held(made);
    let mut files = Map::with_capacity(sizes.len());
    for (_, made) in made {
        match made {
            crate::diff::Made::Spec(value) => {
                total = total.saturating_add(spec(value, max));
                self::files(value, sizes, &mut files);
            }
            crate::diff::Made::Failed(_) => {}
        }
    }
    // Each digest/size map entry owns a digest and a scalar value.
    total.saturating_add(u64::from(files.len()).saturating_mul(ELEMENT.saturating_add(32)))
}
impl super::Side {
    pub(super) fn held_specs(&self) -> u64 {
        held(&self.made, &self.sizes, &self.finding, self.limits.specs)
    }
    pub(crate) fn retain_spec(&mut self, spec: parts::Spec) {
        let target = spec.meta.target.clone();
        self.made.insert(target.clone(), crate::diff::Made::Spec(Box::new(spec))).expect("target capacity");
        if self.specs_before.saturating_add(self.held_specs()) > self.limits.specs {
            self.reject_spec(&target);
        }
    }
    fn reject_spec(&mut self, name: &[u8]) {
        let mut error = super::validation::limit(name);
        let spec = match self.made.get(name) {
            Some(crate::diff::Made::Spec(spec)) => spec,
            Some(crate::diff::Made::Failed(_)) | None => unreachable!("spec holder"),
        };
        let meta = &spec.meta;
        let mut path = skein_lib::Writer::new(
            b"hosts//-.json"
                .len()
                .saturating_add(meta.base.len().saturating_mul(2))
                .saturating_add(meta.environment.len()),
        );
        for piece in [b"hosts/".as_slice(), &meta.base, b"/", &meta.base, b"-", &meta.environment, b".json"] {
            path.put(piece).expect("target path");
        }
        error.at = Some(crate::errors::Place { path: path.finish(), position: None });
        self.errors.push(error);
        self.outcome = self.outcome.worst(crate::vocabulary::Outcome::Failed);
        self.made
            .insert(skein_lib::bytes::copy_of(name), crate::diff::Made::Failed(crate::vocabulary::Class::Configuration))
            .expect("existing target");
    }
    pub(super) fn admit_finding(&mut self, key: Digest) {
        let mut targets = skein_lib::List::with_capacity(self.limits.targets);
        for (name, made) in &self.made {
            if super::comparison::key(Some(made)) == Some(key) {
                targets.push(name.clone()).expect("target bound");
            }
        }
        if let Some(known) = self.finding.record(&key) {
            for target in targets.as_slice() {
                let spec = match self.made.get_mut(target) {
                    Some(crate::diff::Made::Spec(spec)) => spec,
                    Some(crate::diff::Made::Failed(_)) | None => unreachable!("spec holder"),
                };
                let keys = crate::keys::layers(
                    &known.digest,
                    &crate::keys::part(spec, b"packages"),
                    &crate::keys::part(spec, b"software"),
                    &crate::keys::part(spec, b"config"),
                );
                spec.resolution = Some(Box::new(crate::keys::Resolution::Known { key, keys }));
            }
        }
        for target in targets.as_slice() {
            if self.specs_before.saturating_add(self.held_specs()) <= self.limits.specs {
                return;
            }
            self.reject_spec(target);
        }
        self.finding.discard_closure(&key);
    }
}
