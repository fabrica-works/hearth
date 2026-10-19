//! Recorded closures compared by name, then architecture (domain/store.md, 6.4).
use crate::resolution::Package;
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::List;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ResolutionChange {
    Same,
    NewKey,
    Resolved { added: Box<[Listed]>, removed: Box<[Listed]>, versions_changed: Box<[Moved]> },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Listed {
    pub name: Box<[u8]>,
    pub architecture: Box<[u8]>,
    pub version: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Moved {
    pub name: Box<[u8]>,
    pub architecture: Box<[u8]>,
    pub before: Box<[u8]>,
    pub after: Box<[u8]>,
}
#[must_use]
pub fn resolution(from: Digest, to: Digest, before: Option<&[Package]>, after: Option<&[Package]>) -> ResolutionChange {
    if from == to {
        return ResolutionChange::Same;
    }
    let (Some(before), Some(after)) = (before, after) else { return ResolutionChange::NewKey };
    let capacity = u32::try_from(before.len().saturating_add(after.len())).expect("bounded closures");
    let mut added = List::with_capacity(capacity);
    let mut removed = List::with_capacity(capacity);
    let mut versions_changed = List::with_capacity(capacity);
    let mut from = before.iter().peekable();
    let mut to = after.iter().peekable();
    for _ in 0..capacity {
        match (from.peek(), to.peek()) {
            (Some(old), Some(new)) if identity(old) == identity(new) => {
                if old.version != new.version {
                    versions_changed
                        .push(Moved {
                            name: new.name.clone(),
                            architecture: new.architecture.clone(),
                            before: old.version.clone(),
                            after: new.version.clone(),
                        })
                        .expect("closure capacity");
                }
                from.next();
                to.next();
            }
            (Some(old), Some(new)) if identity(old) < identity(new) => {
                removed.push(listed(old)).expect("closure capacity");
                from.next();
            }
            (Some(_) | None, Some(new)) => {
                added.push(listed(new)).expect("closure capacity");
                to.next();
            }
            (Some(old), None) => {
                removed.push(listed(old)).expect("closure capacity");
                from.next();
            }
            (None, None) => break,
        }
    }
    ResolutionChange::Resolved {
        added: added.into_boxed(),
        removed: removed.into_boxed(),
        versions_changed: versions_changed.into_boxed(),
    }
}
fn identity(package: &Package) -> (&[u8], &[u8]) {
    (&package.name, &package.architecture)
}
fn listed(package: &Package) -> Listed {
    Listed { name: package.name.clone(), architecture: package.architecture.clone(), version: package.version.clone() }
}
