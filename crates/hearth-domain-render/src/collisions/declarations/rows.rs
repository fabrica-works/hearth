//! The existing typed rows plus the password reference and implicit own group.
use super::{Claimant, Declared, Rows};
use crate::{
    entries::Located,
    errors::{Collector, Subject},
    kinds::{
        self,
        cmdline::Parameter,
        groups::Group,
        users::{User, UserKind},
    },
    parts::{
        config::{Preset, Tmpfile},
        packages::Preseed,
    },
    revision::declaration::{Map, Name},
};
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
#[derive(Debug)]
pub(super) enum Row {
    User(User, Option<Box<[u8]>>),
    Group(Group, bool),
    Preset(Preset),
    Tmpfile(Tmpfile),
    Preseed(Preseed),
    Parameter(Parameter),
}
impl Row {
    pub(super) const fn subject(&self) -> Subject {
        match self {
            Self::User(..) => Subject::User,
            Self::Group(..) => Subject::Group,
            Self::Preset(_) => Subject::Preset,
            Self::Tmpfile(_) => Subject::Tmpfiles,
            Self::Preseed(_) => Subject::Preseed,
            Self::Parameter(_) => Subject::Parameter,
        }
    }
    pub(super) const fn explicit(&self) -> bool {
        !matches_group(self)
    }
    pub(super) fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::User(a, ap), Self::User(b, bp)) => a.kind == b.kind && ap == bp,
            (Self::Group(_, false), Self::Group(_, false)) => true,
            (Self::Group(a, _), Self::Group(b, _)) => a.gid == b.gid && a.package == b.package,
            (Self::Preset(a), Self::Preset(b)) => a == b,
            (Self::Tmpfile(a), Self::Tmpfile(b)) => a == b,
            (Self::Preseed(a), Self::Preseed(b)) => a == b,
            (Self::Parameter(a), Self::Parameter(b)) => a.order == b.order && a.param == b.param,
            (
                Self::User(..)
                | Self::Group(..)
                | Self::Preset(_)
                | Self::Tmpfile(_)
                | Self::Preseed(_)
                | Self::Parameter(_),
                _,
            ) => false,
        }
    }
}
const fn matches_group(row: &Row) -> bool {
    match row {
        Row::Group(_, explicit) => !*explicit,
        Row::User(..) | Row::Preset(_) | Row::Tmpfile(_) | Row::Preseed(_) | Row::Parameter(_) => false,
    }
}
pub(super) fn gather(rows: &mut Rows, declared: &Declared<'_>, errors: &mut Collector) {
    accounts(rows, declared, errors);
    for map in [Map::Presets, Map::Tmpfiles, Map::Preseeds, Map::Cmdline] {
        for located in &declared.expansion.of(map) {
            let row = match map {
                Map::Presets => {
                    let row = match kinds::presets::entry(located) {
                        Ok(row) => row,
                        Err(error) => {
                            fault(rows, *error, declared, errors);
                            continue;
                        }
                    };
                    Row::Preset(row)
                }
                Map::Tmpfiles => {
                    let row = match kinds::tmpfiles::entry(located) {
                        Ok(row) => row,
                        Err(error) => {
                            fault(rows, *error, declared, errors);
                            continue;
                        }
                    };
                    Row::Tmpfile(row)
                }
                Map::Preseeds => {
                    let Ok(row) = kinds::preseeds::entry(located.entry) else { continue };
                    Row::Preseed(row)
                }
                Map::Cmdline => {
                    let Ok(row) = kinds::cmdline::entry(located.entry) else { continue };
                    Row::Parameter(row)
                }
                Map::Packages
                | Map::Initramfs
                | Map::Software
                | Map::Files
                | Map::Users
                | Map::Groups
                | Map::Data
                | Map::Secrets
                | Map::Identity
                | Map::Live => unreachable!("declaration kinds"),
            };
            add(rows, located, &located.entry.key, row, declared.target, errors);
        }
    }
    let located = declared.expansion.of(Map::Data);
    let mut data = List::with_capacity(u32::try_from(located.len()).expect("expansion bound"));
    for located in &located {
        if let Ok(entry) = kinds::data::entry(located) {
            data.push(entry).expect("expansion bound");
        }
    }
    for line in crate::additions::data_lines(data.as_slice()) {
        rows.insert(
            &line.tmpfile.path.clone(),
            Row::Tmpfile(line.tmpfile),
            Claimant::new(&line.task, line.at),
            declared.target,
            errors,
        );
    }
}
fn add(rows: &mut Rows, located: &Located<'_>, key: &[u8], row: Row, target: &[u8], errors: &mut Collector) {
    rows.insert(key, row, Claimant::new(located.task, located.entry.at), target, errors);
}
fn fault(rows: &mut Rows, mut error: crate::errors::Error, declared: &Declared<'_>, errors: &mut Collector) {
    rows.valid = false;
    error.target = Some(copy_of(declared.target));
    errors.push(error);
}

fn accounts(rows: &mut Rows, declared: &Declared<'_>, errors: &mut Collector) {
    let users = declared.expansion.of(Map::Users);
    let mut passwords = skein_lib::Map::with_capacity(rows.rows.capacity());
    for located in &users {
        if let Some(password) = located.entry.text(Name::Password)
            && passwords.insert((located.task, located.entry.key.as_ref()), password).is_err()
        {
            rows.valid = false;
            errors.push(super::error(
                declared.target,
                crate::errors::Kind::Evaluation(crate::evaluate::FaultKind::Limit),
                &Claimant::new(located.task, located.entry.at),
                None,
            ));
        }
    }
    for user in &declared.accounts.users {
        let mut password = None;
        if let Some(secret) = passwords.get(&(user.task.as_ref(), user.name.as_ref())) {
            password = Some(copy_of(secret));
        }
        rows.insert(
            &user.name,
            Row::User(user.clone(), password),
            Claimant::new(&user.task, user.at),
            declared.target,
            errors,
        );
        if let UserKind::Sysusers { uid, group, .. } = &user.kind
            && *group == user.name
        {
            rows.insert(
                &user.name,
                Row::Group(
                    Group { name: user.name.clone(), task: user.task.clone(), at: user.at, gid: *uid, package: false },
                    false,
                ),
                Claimant::new(&user.task, user.at),
                declared.target,
                errors,
            );
        }
    }
    for group in &declared.accounts.groups {
        rows.insert(
            &group.name,
            Row::Group(group.clone(), true),
            Claimant::new(&group.task, group.at),
            declared.target,
            errors,
        );
    }
}
