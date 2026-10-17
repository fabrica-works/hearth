//! A resolution run's decoded install lines and classified cause (render.md, 5.5).
use crate::{errors::Resolution, limits::Limits, vocabulary::Class};
use alloc::boxed::Box;
use skein_lib::Map;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Inst {
    pub name: Box<[u8]>,
    pub version: Box<[u8]>,
    pub architecture: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Index {
    pub path: Box<[u8]>,
    pub size: u64,
    pub sha256: hearth_lib::encode::Digest,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Line {
    pub indices: Box<[Index]>,
    pub uri: Box<[u8]>,
    pub suite: Box<[u8]>,
    pub components: Box<[Box<[u8]>]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tools {
    pub apt: Box<[u8]>,
    pub mmdebstrap: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Costs {
    pub fetched: u64,
    pub took: skein_lib::Duration,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Cause {
    Fetch,
    Resolve,
    Space,
    Absent { uri: Box<[u8]>, suite: Box<[u8]> },
    Unknown(Box<[u8]>),
    Unsatisfiable(Option<Box<[u8]>>),
    Other,
}
impl Cause {
    #[must_use]
    pub const fn class(&self) -> Class {
        match self {
            Self::Fetch | Self::Resolve | Self::Space => Class::Operational,
            Self::Absent { .. } | Self::Unknown(_) | Self::Unsatisfiable(_) | Self::Other => Class::Configuration,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Stanza {
    pub package: Box<[u8]>,
    pub version: Box<[u8]>,
    pub architecture: Box<[u8]>,
    pub filename: Box<[u8]>,
    pub size: u64,
    pub sha256: hearth_lib::encode::Digest,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Package {
    pub name: Box<[u8]>,
    pub architecture: Box<[u8]>,
    pub version: Box<[u8]>,
    pub source: u32,
    pub filename: Box<[u8]>,
    pub size: u64,
    pub sha256: hearth_lib::encode::Digest,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Triple {
    name: Box<[u8]>,
    architecture: Box<[u8]>,
    version: Box<[u8]>,
}
/// Only installed triples are retained; indices may contain unrelated stanzas.
#[derive(Debug)]
pub struct Closure {
    packages: Map<Triple, Option<Package>>,
    max: u32,
}
impl Closure {
    #[must_use]
    pub fn new(limits: &Limits) -> Self {
        Self { packages: Map::with_capacity(limits.closure), max: limits.closure }
    }
    pub fn inst(&mut self, inst: Inst) -> Result<(), Resolution> {
        let key = Triple { name: inst.name, architecture: inst.architecture, version: inst.version };
        if !self.packages.contains_key(&key) && self.packages.insert(key, None).is_err() {
            return Err(Resolution::TooMany { max: self.max });
        }
        Ok(())
    }
    pub fn stanza(&mut self, line: u32, stanza: Stanza) -> Result<(), Resolution> {
        let key = Triple { name: stanza.package, architecture: stanza.architecture, version: stanza.version };
        let Some(package) = self.packages.get_mut(&key) else { return Ok(()) };
        if let Some(first) = package {
            if first.size != stanza.size || first.sha256 != stanza.sha256 {
                return Err(Resolution::Disagree {
                    name: key.name,
                    architecture: key.architecture,
                    version: key.version,
                });
            }
            if first.source <= line {
                return Ok(());
            }
        }
        *package = Some(Package {
            name: key.name,
            architecture: key.architecture,
            version: key.version,
            source: line,
            filename: stanza.filename,
            size: stanza.size,
            sha256: stanza.sha256,
        });
        Ok(())
    }
    pub fn close(self, names: &[Box<[u8]>]) -> Result<Box<[Package]>, Resolution> {
        let mut closure = skein_lib::List::with_capacity(self.max);
        for (key, package) in &self.packages {
            let Some(package) = package else {
                return Err(Resolution::Unmatched {
                    name: key.name.clone(),
                    architecture: key.architecture.clone(),
                    version: key.version.clone(),
                });
            };
            closure.push(package.clone()).expect("bounded installed triples");
        }
        for name in names {
            let mut installed = false;
            for package in &closure {
                installed |= &package.name == name;
            }
            if !installed {
                return Err(Resolution::Missing { package: name.clone() });
            }
        }
        Ok(closure.into_boxed())
    }
}
