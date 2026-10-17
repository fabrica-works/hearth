//! Ordered, bounded claims, retaining each declaration's place.
use super::{Declared, files};
use crate::{entries::Located, kinds, revision::declaration::Map as KindMap};
use crate::{
    errors::{Class, Collector, Error, Kind, Place, Subject},
    limits::Limits,
};
use alloc::boxed::Box;
use skein_lib::Writer;
use skein_lib::{Map, bytes::copy_of};
#[derive(Debug)]
pub(super) struct Claim {
    pub task: Box<[u8]>,
    pub path: Box<[u8]>,
    pub at: Place,
    pub subject: Subject,
    pub name: Option<(Subject, Box<[u8]>)>,
    pub tar: bool,
}
type Key = (Box<[u8]>, Box<[u8]>, Subject, Box<[u8]>, bool);
pub(super) fn gather(
    declared: &Declared<'_>,
    limits: &Limits,
    errors: &mut Collector,
    valid: &mut bool,
) -> Map<Key, Claim> {
    let mut claims = Map::with_capacity(limits.expansion);
    for plan in declared.plans {
        for install in &plan.installs {
            let path = match (&install.make, install.at) {
                (files::Make::Rendered { template, .. }, None) => template.clone(),
                (_, None) => joined(&[b"tasks/", &plan.task, b"/tree", &install.path]),
                (_, Some(_)) => declaration(&plan.task),
            };
            insert(
                &mut claims,
                &install.path,
                Claim {
                    task: plan.task.clone(),
                    path: install.path.clone(),
                    at: Place { path, position: install.at },
                    subject: Subject::Tree,
                    name: None,
                    tar: false,
                },
                declared.target,
                errors,
                valid,
            );
        }
    }
    for kind in [KindMap::Data, KindMap::Secrets, KindMap::Identity, KindMap::Software] {
        for row in &declared.expansion.of(kind) {
            let claim = match kind {
                KindMap::Data => {
                    let Ok(entry) = kinds::data::entry(row) else { continue };
                    located(row, Subject::DataPath, entry.path, Some(Subject::DataName), false)
                }
                KindMap::Secrets => {
                    let Ok(entry) = kinds::secrets::file(row) else { continue };
                    located(row, Subject::Secret, entry.path, Some(Subject::Secret), false)
                }
                KindMap::Identity => {
                    let Ok(entry) = kinds::identity::entry(row) else { continue };
                    if let Some(path) = entry.public_path {
                        insert(
                            &mut claims,
                            &row.entry.key,
                            located(row, Subject::Identity, path, None, false),
                            declared.target,
                            errors,
                            valid,
                        );
                    }
                    located(row, Subject::Identity, entry.path, Some(Subject::Identity), false)
                }
                KindMap::Software => {
                    let Ok((entry, _fetch)) = kinds::software::entry(row) else { continue };
                    let tar = match entry.how {
                        crate::parts::software::How::Tar { .. } => true,
                        crate::parts::software::How::File { .. } => false,
                    };
                    located(row, Subject::Download, entry.path, None, tar)
                }
                KindMap::Packages
                | KindMap::Initramfs
                | KindMap::Preseeds
                | KindMap::Cmdline
                | KindMap::Files
                | KindMap::Users
                | KindMap::Groups
                | KindMap::Presets
                | KindMap::Tmpfiles
                | KindMap::Live => unreachable!("claim kinds"),
            };
            insert(&mut claims, &row.entry.key, claim, declared.target, errors, valid);
        }
    }
    claims
}
fn located(row: &Located<'_>, subject: Subject, path: Box<[u8]>, name: Option<Subject>, tar: bool) -> Claim {
    let mut claim = Claim {
        task: copy_of(row.task),
        path,
        at: Place { path: declaration(row.task), position: Some(row.entry.at) },
        subject,
        name: None,
        tar,
    };
    if let Some(subject) = name {
        claim.name = Some((subject, row.entry.key.clone()));
    }
    claim
}

fn insert(
    claims: &mut Map<Key, Claim>,
    key: &[u8],
    claim: Claim,
    target: &[u8],
    errors: &mut Collector,
    valid: &mut bool,
) {
    let key = (claim.task.clone(), copy_of(key), claim.subject, claim.path.clone(), claim.name.is_some());
    if let Err((_key, claim)) = claims.insert(key, claim) {
        errors.push(Error {
            side: None,
            kind: Kind::Evaluation(crate::evaluate::FaultKind::Limit),
            class: Class::Configuration,
            at: Some(claim.at),
            also: None,
            target: Some(copy_of(target)),
            task: Some(claim.task),
        });
        *valid = false;
    }
}
fn declaration(task: &[u8]) -> Box<[u8]> {
    joined(&[b"tasks/", task, b"/task.json"])
}
fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.saturating_add(part.len());
    }
    let mut path = Writer::new(length);
    for part in parts {
        path.put(part).expect("path capacity");
    }
    path.finish()
}
