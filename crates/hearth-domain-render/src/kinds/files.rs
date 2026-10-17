//! One source per destination and directory governance (format.md, 5.5).
use crate::{
    entries::{Concrete, Fault, Located},
    errors::{Format, Kind, Repository, Schema, TypeName},
    revision::{
        Oid, Position,
        declaration::{Declaration, Interpolated, Map, Name},
    },
    schema::forms,
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::{List, Map as Ordered, bytes::copy_of};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Plan {
    pub task: Box<[u8]>,
    pub installs: Box<[Install]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Install {
    pub path: Box<[u8]>,
    pub at: Option<Position>,
    pub make: Make,
    pub owner: Box<[u8]>,
    pub group: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Make {
    Static { sha256: Digest, mode: u16 },
    Link { target: Box<[u8]> },
    Directory { mode: u16 },
    Rendered { template: Box<[u8]>, mode: u16 },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TreeFile {
    pub path: Box<[u8]>,
    pub source: Source,
    pub mode: u16,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Source {
    Static { sha256: Digest },
    Template { oid: Oid, path: Box<[u8]> },
    Link { target: Box<[u8]> },
}
#[must_use]
pub fn link(target: &[u8]) -> bool {
    !target.is_empty() && target.len() <= 4095 && !target.contains(&0)
}
#[must_use]
pub fn dropped(declaration: &Declaration, kept: &[Located<'_>]) -> Box<[Box<[u8]>]> {
    let mut capacity = 0_usize;
    for group in &declaration.entries {
        capacity = capacity.saturating_add(group.entries.len());
    }
    let mut paths = List::with_capacity(u32::try_from(capacity).expect("declaration bound"));
    for group in &declaration.entries {
        if group.map != Map::Files {
            continue;
        }
        for entry in &group.entries {
            if entry.for_each.is_some() {
                continue;
            }
            match &entry.key {
                Interpolated::Plain { text, .. } if !contains(kept, text) => {
                    paths.push(text.clone()).expect("declaration bound");
                }
                Interpolated::Plain { .. } | Interpolated::Parts { .. } => {}
            }
        }
    }
    paths.into_boxed()
}
pub fn plan(
    task: &[u8],
    tree: &[TreeFile],
    files: &[Located<'_>],
    dropped: &[Box<[u8]>],
    faults: &mut List<Fault>,
) -> Plan {
    let capacity = u32::try_from(tree.len().saturating_add(files.len())).expect("listing and expansion bound");
    let mut installs = Ordered::with_capacity(capacity);
    for file in tree {
        if excluded(&file.path, dropped) {
            continue;
        }
        let make = match &file.source {
            Source::Static { sha256 } => Make::Static { sha256: *sha256, mode: file.mode },
            Source::Template { path, .. } => Make::Rendered { template: path.clone(), mode: file.mode },
            Source::Link { target } => Make::Link { target: target.clone() },
        };
        installs
            .insert(
                file.path.clone(),
                Install { path: file.path.clone(), at: None, make, owner: copy_of(b"root"), group: copy_of(b"root") },
            )
            .expect("plan capacity");
    }
    for row in files {
        if row.task != task || excluded(&row.entry.key, dropped) {
            continue;
        }
        match install(row.entry, tree) {
            Ok(install) => {
                installs.insert(install.path.clone(), install).expect("plan capacity");
            }
            Err(fault) => faults.push(fault).expect("fault capacity"),
        }
    }
    let mut result = List::with_capacity(capacity);
    for (_, install) in &installs {
        let mut install = install.clone();
        install.owner = owner(&install.path, files, Name::Owner);
        install.group = owner(&install.path, files, Name::Group);
        result.push(install).expect("plan capacity");
    }
    Plan { task: copy_of(task), installs: result.into_boxed() }
}
fn excluded(path: &[u8], dropped: &[Box<[u8]>]) -> bool {
    for key in dropped {
        if key.as_ref() == path || (key.ends_with(b"/") && path.starts_with(key)) {
            return true;
        }
    }
    false
}
fn owner(path: &[u8], rows: &[Located<'_>], field: Name) -> Box<[u8]> {
    let mut length = 0;
    let mut value = b"root".as_slice();
    for row in rows {
        let key = row.entry.key.as_ref();
        if key.len() >= length
            && (key.strip_suffix(b"/").unwrap_or(key) == path || (key.ends_with(b"/") && path.starts_with(key)))
            && let Some(given) = row.entry.text(field)
        {
            length = key.len();
            value = given;
        }
    }
    copy_of(value)
}
fn install(entry: &Concrete, tree: &[TreeFile]) -> Result<Install, Fault> {
    let path = entry.key.as_ref();
    let directory = path.ends_with(b"/");
    let checked = if directory { path.strip_suffix(b"/").expect("suffix") } else { path };
    if !forms::check(Format::Path, checked) {
        return Err(schema(Schema::Malformed { format: Format::Path }, entry.at));
    }
    let mut source = None;
    for field in &entry.fields {
        if [Name::Template, Name::Copy, Name::Link, Name::Directory].contains(&field.name) {
            if directory {
                return Err(undeclared(field.name, field.at));
            }
            if let Some(first) = source {
                return Err(schema(Schema::Source { first, second: field.name }, field.at));
            }
            source = Some(field.name);
        }
    }
    let mut tree_file = None;
    let mut subtree = false;
    for file in tree {
        if file.path.as_ref() == path {
            tree_file = Some(file);
        }
        subtree |= file.path.starts_with(path);
    }
    let make = match source {
        Some(Name::Directory) => {
            if entry.bool(Name::Directory) != Some(true) {
                return Err(schema(Schema::Wrong { expected: TypeName::Bool }, field_at(entry, Name::Directory)));
            }
            Make::Directory { mode: mode(entry, 0o755)? }
        }
        Some(Name::Link) => {
            if let Some(field) = field(entry, Name::Mode) {
                return Err(undeclared(Name::Mode, field.at));
            }
            let target = entry.text(Name::Link).expect("typed link");
            if !link(target) {
                return Err(schema(Schema::Malformed { format: Format::Link }, field_at(entry, Name::Link)));
            }
            Make::Link { target: copy_of(target) }
        }
        Some(Name::Copy | Name::Template) => return Err(repository(Repository::NoTree, entry.at)),
        Some(
            Name::Action
            | Name::Age
            | Name::Algorithm
            | Name::Argument
            | Name::Base
            | Name::Class
            | Name::Component
            | Name::Count
            | Name::From
            | Name::Gecos
            | Name::Gid
            | Name::Group
            | Name::Groups
            | Name::Home
            | Name::Kind
            | Name::Mode
            | Name::Order
            | Name::Owner
            | Name::Param
            | Name::Password
            | Name::Path
            | Name::Paths
            | Name::PublicPath
            | Name::Sha256
            | Name::Shell
            | Name::Strip
            | Name::Subordinate
            | Name::Type
            | Name::Uid
            | Name::Unit
            | Name::Unpack
            | Name::Url
            | Name::Value,
        ) => unreachable!("source names"),
        None if directory && subtree => Make::Directory { mode: mode(entry, 0o755)? },
        None => match tree_file {
            Some(TreeFile { source: Source::Static { sha256 }, mode: default, .. }) => {
                Make::Static { sha256: *sha256, mode: mode(entry, *default)? }
            }
            Some(TreeFile { source: Source::Template { path, .. }, mode: default, .. }) => {
                Make::Rendered { template: path.clone(), mode: mode(entry, *default)? }
            }
            Some(TreeFile { source: Source::Link { target }, .. }) => {
                if let Some(field) = field(entry, Name::Mode) {
                    return Err(undeclared(Name::Mode, field.at));
                }
                Make::Link { target: target.clone() }
            }
            None => return Err(repository(Repository::NoTree, entry.at)),
        },
    };
    if source.is_some() && tree_file.is_some() {
        return Err(repository(Repository::Destination, entry.at));
    }
    Ok(Install { path: copy_of(checked), at: Some(entry.at), make, owner: copy_of(b"root"), group: copy_of(b"root") })
}
fn mode(entry: &Concrete, default: u16) -> Result<u16, Fault> {
    let Some(text) = entry.text(Name::Mode) else {
        return Ok(default);
    };
    if !forms::check(Format::Mode, text) {
        return Err(schema(Schema::Malformed { format: Format::Mode }, field_at(entry, Name::Mode)));
    }
    let mut mode = 0_u16;
    for byte in text {
        mode = mode
            .checked_mul(8)
            .expect("mode bound")
            .checked_add(u16::from(byte.saturating_sub(b'0')))
            .expect("checked mode");
    }
    Ok(mode)
}
fn schema(kind: Schema, at: Position) -> Fault {
    Fault { kind: Kind::Schema(kind), at, also: None }
}
fn repository(kind: Repository, at: Position) -> Fault {
    Fault { kind: Kind::Repository(kind), at, also: None }
}
fn undeclared(name: Name, at: Position) -> Fault {
    schema(Schema::UndeclaredField { field: copy_of(name.key()), declared: Box::new([]) }, at)
}

fn contains(rows: &[Located<'_>], key: &[u8]) -> bool {
    for row in rows {
        if row.entry.key.as_ref() == key {
            return true;
        }
    }
    false
}
fn field(entry: &Concrete, name: Name) -> Option<&crate::entries::Made> {
    let mut found = None;
    for field in &entry.fields {
        if field.name == name {
            found = Some(field);
        }
    }
    found
}

fn field_at(entry: &Concrete, name: Name) -> Position {
    field(entry, name).expect("present field").at
}
