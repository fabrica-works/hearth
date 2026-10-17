//! An independent, deliberately simple closure for the drawn acyclic universes.
use hearth_domain_render::{
    keys::Keys,
    parts::{PartRecord, Spec},
};
use hearth_fake_machine::hearth::universe::{Package, Universe};
use hearth_lib::{encode::Digest, hash::H};
use std::collections::BTreeMap;

pub fn closure<'a>(universe: &'a Universe, include: &[u8]) -> Vec<&'a Package> {
    let mut closed = BTreeMap::new();
    let mut pending: Vec<_> = universe
        .packages()
        .iter()
        .filter(|package| package.essential || package.name.as_ref() == b"apt" || package.name.as_ref() == include)
        .collect();
    for _ in 0..universe.packages().len().saturating_mul(universe.packages().len()) {
        let Some(package) = pending.pop() else {
            return closed.into_values().collect();
        };
        if closed.insert(package.name.as_ref(), package).is_some() {
            continue;
        }
        for group in package.pre_depends.iter().chain(package.depends.iter()) {
            let first = group.iter().next().expect("a first dependency alternative");
            let provider = universe
                .packages()
                .iter()
                .filter(|package| package.provides.iter().any(|provide| provide.name == first.name))
                .min_by_key(|package| (!package.essential, package.priority));
            pending.push(universe.package(&first.name).or(provider).expect("drawn dependency has a candidate"));
        }
    }
    unreachable!("drawn acyclic dependencies close")
}
pub fn digest(packages: &[&Package]) -> Digest {
    let mut hash = H::new(b"closure", 1);
    hash.values().list(packages.len() as u64);
    for package in packages {
        hash.values().list(4);
        hash.values().bytes(&package.name);
        hash.values().bytes(&package.architecture);
        hash.values().bytes(&package.version);
        hash.values().digest(&Digest(package.sha256));
    }
    hash.finish()
}
pub fn layers(digest: Digest, spec: &Spec) -> Keys {
    let mut parent = digest;
    let mut layers = BTreeMap::new();
    for tag in [b"packages".as_slice(), b"software", b"config"] {
        let part = spec
            .parts
            .iter()
            .find(|part| match &part.record {
                PartRecord::Packages(_) => tag == b"packages",
                PartRecord::Software(_) => tag == b"software",
                PartRecord::Config(_) => tag == b"config",
                PartRecord::Boot(_)
                | PartRecord::Data(_)
                | PartRecord::Exports(_)
                | PartRecord::Live(_)
                | PartRecord::Secrets(_) => false,
            })
            .expect("one of each layer");
        let mut hash = H::new(tag, 2);
        hash.values().digest(&parent);
        hash.values().digest(&part.digest);
        parent = hash.finish();
        layers.insert(tag, parent);
    }
    Keys {
        packages: layers[b"packages".as_slice()],
        software: layers[b"software".as_slice()],
        config: layers[b"config".as_slice()],
    }
}
