//! Target discovery and whole-placement replacement (format.md, 3.1, 3.4).
use crate::{
    limits::SHIPPED,
    revision::{self, Class, Entry},
    vocabulary::{As, Blob},
};
use alloc::boxed::Box;
use skein_lib::{List, Map, bytes};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Target {
    pub name: Box<[u8]>,
    pub base: Box<[u8]>,
    pub environment: Box<[u8]>,
    pub documents: [Blob; 2],
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TooMany;
pub fn targets(entries: &[Entry]) -> Result<Box<[Target]>, TooMany> {
    let capacity = u32::try_from(entries.len()).expect("listing's configured capacity");
    let mut environments = Map::with_capacity(capacity);
    for entry in entries {
        match revision::class(&entry.path) {
            Class::Environment { env } => {
                environments.insert(bytes::copy_of(env), entry).expect("listing capacity");
            }
            Class::Fleet | Class::Host { .. } | Class::Other => {}
        }
    }
    let mut discovered = Map::with_capacity(SHIPPED.targets);
    for entry in entries {
        match revision::class(&entry.path) {
            Class::Host { base, name } => {
                if name == base {
                    continue;
                }
                let Some(suffix) = name.strip_prefix(base) else {
                    continue;
                };
                let Some(env) = suffix.strip_prefix(b"-") else {
                    continue;
                };
                let Some(environment) = environments.get(env) else {
                    continue;
                };
                let target = Target {
                    name: bytes::copy_of(name),
                    base: bytes::copy_of(base),
                    environment: bytes::copy_of(env),
                    documents: [blob(environment), blob(entry)],
                };
                if discovered.insert(target.name.clone(), target).is_err() {
                    return Err(TooMany);
                }
            }
            Class::Fleet | Class::Environment { .. } | Class::Other => {}
        }
    }
    let mut targets = List::with_capacity(discovered.len());
    for (_, target) in &discovered {
        targets.push(target.clone()).expect("discovered count");
    }
    Ok(targets.into_boxed())
}
fn blob(entry: &Entry) -> Blob {
    Blob { oid: entry.oid.clone(), size: entry.size, kind: As::Document }
}
