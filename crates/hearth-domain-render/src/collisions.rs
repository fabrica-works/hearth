//! Claims meet before templates run (domain/render.md, 3.7).
mod claims;
mod declarations;
mod ids;
use crate::{
    entries::Expansion,
    errors::{Class, Collector, Collision, Error, Kind, Subject},
    kinds::{files, users::Accounts},
    limits::Limits,
};
use alloc::boxed::Box;
use claims::Claim;
pub use declarations::{Survivors, declarations};
pub use ids::ids;
use skein_lib::{Map, bytes::copy_of};
/// Everything the target's tasks declared.
#[derive(Debug)]
pub struct Declared<'a> {
    pub target: &'a [u8],
    pub expansion: &'a Expansion,
    pub plans: &'a [files::Plan],
    pub accounts: &'a Accounts,
}
/// Refuses tree, data, secret, identity and download collisions.
/// Returns whether every claim fits and agrees with the earlier claims.
#[must_use]
pub fn paths(declared: &Declared<'_>, limits: &Limits, errors: &mut Collector) -> bool {
    let mut valid = true;
    let claims = claims::gather(declared, limits, errors, &mut valid);
    let mut paths: Map<Box<[u8]>, &Claim> = Map::with_capacity(limits.expansion);
    let mut names: Map<(Subject, Box<[u8]>), &Claim> = Map::with_capacity(limits.expansion);
    for (_, claim) in &claims {
        if let Some((subject, name)) = &claim.name {
            let key = (*subject, name.clone());
            if let Some(first) = names.get(&key) {
                report(declared.target, given(*subject, name, first, claim), first, claim, errors);
                valid = false;
            } else {
                names.insert(key, claim).expect("claims bound");
            }
        }
        if let Some(first) = paths.get(&claim.path) {
            pair(declared.target, first, claim, errors, &mut valid);
            continue;
        }
        // Every ancestor ends at a slash; a lexical prefix alone is not a path ancestor.
        for (index, byte) in claim.path.iter().enumerate().skip(1) {
            if *byte == b'/'
                && let Some(first) = paths.get(claim.path.get(..index).expect("path index"))
            {
                pair(declared.target, first, claim, errors, &mut valid);
            }
        }
        for (path, first) in paths.range(claim.path.clone()..) {
            if !path.starts_with(claim.path.as_ref()) {
                break;
            }
            if beneath(path, &claim.path) {
                pair(declared.target, first, claim, errors, &mut valid);
            }
        }
        paths.insert(claim.path.clone(), claim).expect("claims bound");
    }
    valid
}
fn beneath(path: &[u8], parent: &[u8]) -> bool {
    match path.strip_prefix(parent) {
        Some(rest) => rest.starts_with(b"/"),
        None => false,
    }
}
fn given(subject: Subject, name: &[u8], first: &Claim, second: &Claim) -> Collision {
    Collision::Given { subject, name: copy_of(name), first: first.task.clone(), second: second.task.clone() }
}
fn pair(target: &[u8], first: &Claim, second: &Claim, errors: &mut Collector, valid: &mut bool) {
    let equal = first.path == second.path;
    let first_data = first.subject == Subject::DataPath;
    let second_data = second.subject == Subject::DataPath;
    let (kind, at, also) = if first_data && second_data {
        (
            Collision::Overlap { subject: Subject::DataPath, first: first.path.clone(), second: second.path.clone() },
            first,
            second,
        )
    } else if (first_data && (equal || beneath(&second.path, &first.path)))
        || (second_data && (equal || beneath(&first.path, &second.path)))
    {
        let (data, hidden) = if first_data { (first, second) } else { (second, first) };
        (
            Collision::Hidden {
                path: hidden.path.clone(),
                data: data.path.clone(),
                first: data.task.clone(),
                second: hidden.task.clone(),
            },
            data,
            hidden,
        )
    } else if equal {
        (
            given(
                if first.subject == Subject::Tree || second.subject == Subject::Tree {
                    Subject::Tree
                } else {
                    second.subject
                },
                &first.path,
                first,
                second,
            ),
            first,
            second,
        )
    } else if (first.tar && beneath(&second.path, &first.path)) || (second.tar && beneath(&first.path, &second.path)) {
        let (tar, child) =
            if first.tar && beneath(&second.path, &first.path) { (first, second) } else { (second, first) };
        (
            Collision::Overlap { subject: Subject::Download, first: tar.path.clone(), second: child.path.clone() },
            first,
            second,
        )
    } else {
        return;
    };
    report(target, kind, at, also, errors);
    *valid = false;
}
fn report(target: &[u8], kind: Collision, at: &Claim, also: &Claim, errors: &mut Collector) {
    errors.push(Error {
        side: None,
        kind: Kind::Collision(kind),
        class: Class::Configuration,
        at: Some(at.at.clone()),
        also: Some(also.at.clone()),
        target: Some(copy_of(target)),
        task: None,
    });
}
