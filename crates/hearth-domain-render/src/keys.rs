//! Resolution and layer keys (render.md, 4.5, 5.1).
use crate::parts::{PartRecord, Spec, packages::Inputs};
use hearth_lib::{encode::Digest, hash::H};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Keys {
    pub packages: Digest,
    pub software: Digest,
    pub config: Digest,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Resolution {
    Needed { key: Digest },
    Known { key: Digest, keys: Keys },
}
impl Resolution {
    #[must_use]
    pub const fn key(&self) -> Digest {
        match self {
            Self::Needed { key } | Self::Known { key, .. } => *key,
        }
    }
}
#[must_use]
pub fn resolution(inputs: &Inputs) -> Digest {
    let mut hash = H::new(b"resolution", 4);
    for items in [&inputs.sources, &inputs.request, &inputs.variant, &inputs.apt] {
        for item in items {
            hash.values().item(item);
        }
    }
    hash.finish()
}
#[must_use]
pub fn layers(resolution: &Digest, packages: &Digest, software: &Digest, config: &Digest) -> Keys {
    let packages = layer(b"packages", resolution, packages);
    let software = layer(b"software", &packages, software);
    Keys { packages, software, config: layer(b"config", &software, config) }
}
fn layer(tag: &[u8], parent: &Digest, part: &Digest) -> Digest {
    let mut hash = H::new(tag, 2);
    hash.values().digest(parent);
    hash.values().digest(part);
    hash.finish()
}
#[must_use]
pub fn part(spec: &Spec, name: &[u8]) -> Digest {
    for part in &spec.parts {
        let tag: &[u8] = match &part.record {
            PartRecord::Data(_) => b"data",
            PartRecord::Config(_) => b"config",
            PartRecord::Packages(_) => b"packages",
            PartRecord::Secrets(_) => b"secrets",
            PartRecord::Software(_) => b"software",
            PartRecord::Live(_) => b"live",
            PartRecord::Boot(_) => b"boot",
            PartRecord::Exports(_) => b"exports",
        };
        if tag == name {
            return part.digest;
        }
    }
    unreachable!("every output part is assembled")
}

#[must_use]
pub fn closure(packages: &[crate::resolution::Package]) -> Digest {
    let mut hash = H::new(b"closure", 1);
    hash.values().list(u64::try_from(packages.len()).expect("bounded closure"));
    for package in packages {
        hash.values().list(4);
        hash.values().bytes(&package.name);
        hash.values().bytes(&package.architecture);
        hash.values().bytes(&package.version);
        hash.values().digest(&package.sha256);
    }
    hash.finish()
}
