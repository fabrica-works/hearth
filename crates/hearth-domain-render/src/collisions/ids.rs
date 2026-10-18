//! Sort account ids and ranges once, then compare each neighbouring run.
use super::claims;
use crate::{
    errors::{Class, Collector, Collision, Error, Id, Kind, Place, Subject},
    kinds::users::{Accounts, User, UserKind},
    limits::Limits,
    revision::Position,
};
use alloc::boxed::Box;
use skein_lib::{Map, bytes::copy_of};
#[derive(Debug)]
struct Claim<'a> {
    task: &'a [u8],
    name: &'a [u8],
    at: Position,
    own: bool,
}
type Key = (u32, Box<[u8]>, Box<[u8]>);
/// Refuses shared ids and intersecting half-open subordinate ranges.
#[must_use]
pub fn ids(accounts: &Accounts, target: &[u8], limits: &Limits, errors: &mut Collector) -> bool {
    let mut valid = true;
    let mut uids = Map::with_capacity(limits.expansion);
    let mut gids = Map::with_capacity(limits.expansion);
    let mut ranges = Map::with_capacity(limits.expansion);
    for user in &accounts.users {
        let (uid, gid, own) = match &user.kind {
            UserKind::Sysusers { uid, group, .. } => (*uid, if *group == user.name { *uid } else { None }, true),
            UserKind::Package { uid, gid } => (*uid, *gid, false),
        };
        if let Some(uid) = uid {
            insert(&mut uids, uid, claim(user, false), target, errors, &mut valid);
        }
        if let Some(gid) = gid {
            insert(&mut gids, gid, claim(user, own), target, errors, &mut valid);
        }
        if let UserKind::Sysusers { subordinate: Some((base, count)), .. } = user.kind {
            let key = (base, user.task.clone(), user.name.clone());
            if ranges.insert(key, (base.checked_add(count).expect("checked range"), claim(user, false))).is_err() {
                limit(target, &claim(user, false), errors);
                valid = false;
            }
        }
    }
    for group in &accounts.groups {
        if let Some(gid) = group.gid {
            insert(
                &mut gids,
                gid,
                Claim { task: &group.task, name: &group.name, at: group.at, own: false },
                target,
                errors,
                &mut valid,
            );
        }
    }
    shared(&uids, Id::Uid, target, errors, &mut valid);
    shared(&gids, Id::Gid, target, errors, &mut valid);
    let mut widest: Option<(u32, &Claim<'_>)> = None;
    for ((base, _, _), (end, who)) in &ranges {
        if let Some((right, first)) = widest {
            if *base < right {
                let (at, also) = ordered(first, who);
                push(
                    target,
                    Collision::Overlap { subject: Subject::Range, first: copy_of(at.name), second: copy_of(also.name) },
                    at,
                    also,
                    errors,
                );
                valid = false;
            }
            if *end > right {
                widest = Some((*end, who));
            }
        } else {
            widest = Some((*end, who));
        }
    }
    valid
}
fn claim(user: &User, own: bool) -> Claim<'_> {
    Claim { task: &user.task, name: &user.name, at: user.at, own }
}
fn insert<'a>(
    map: &mut Map<Key, Claim<'a>>,
    value: u32,
    claim: Claim<'a>,
    target: &[u8],
    errors: &mut Collector,
    valid: &mut bool,
) {
    let key = (value, copy_of(claim.task), copy_of(claim.name));
    if let Err((_key, claim)) = map.insert(key, claim) {
        limit(target, &claim, errors);
        *valid = false;
    }
}
fn shared(map: &Map<Key, Claim<'_>>, id: Id, target: &[u8], errors: &mut Collector, valid: &mut bool) {
    let mut first: Option<(u32, &Claim<'_>)> = None;
    for ((value, _, _), claim) in map {
        if let Some((number, at)) = first
            && number == *value
        {
            if at.name != claim.name && !(at.own && claim.own) {
                push(
                    target,
                    Collision::Shared { id, value: *value, first: copy_of(at.name), second: copy_of(claim.name) },
                    at,
                    claim,
                    errors,
                );
                *valid = false;
            }
            // A declared or package gid must also meet subsequent own groups.
            if at.own && !claim.own {
                first = Some((*value, claim));
            }
        } else {
            first = Some((*value, claim));
        }
    }
}
fn ordered<'a, 'b>(first: &'a Claim<'b>, second: &'a Claim<'b>) -> (&'a Claim<'b>, &'a Claim<'b>) {
    if (first.task, first.name) <= (second.task, second.name) { (first, second) } else { (second, first) }
}
fn place(claim: &Claim<'_>) -> Place {
    Place { path: claims::declaration(claim.task), position: Some(claim.at) }
}
fn push(target: &[u8], collision: Collision, at: &Claim<'_>, also: &Claim<'_>, errors: &mut Collector) {
    errors.push(Error {
        side: None,
        kind: Kind::Collision(collision),
        class: Class::Configuration,
        at: Some(place(at)),
        also: Some(place(also)),
        target: Some(copy_of(target)),
        task: None,
    });
}
fn limit(target: &[u8], claim: &Claim<'_>, errors: &mut Collector) {
    errors.push(Error {
        side: None,
        kind: Kind::Evaluation(crate::evaluate::FaultKind::Limit),
        class: Class::Configuration,
        at: Some(place(claim)),
        also: None,
        target: Some(copy_of(target)),
        task: Some(copy_of(claim.task)),
    });
}
