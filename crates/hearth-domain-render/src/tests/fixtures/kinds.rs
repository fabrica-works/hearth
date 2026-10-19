use crate::{
    entries::{Concrete, Fault, Fielded, Located, Made},
    kinds::files::{self, Plan, Source, TreeFile},
    revision::{Position, declaration::Name},
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::{List, bytes::copy_of};
pub const FILE_AT: Position = Position { line: 3, column: 5 };
pub const FIELD_AT: Position = Position { line: 3, column: 20 };
// protocol/format.md, 5.5: PATH_MAX less its terminating NUL.
pub const LINK_MAX: usize = 4095;
pub fn tree(paths: &[&[u8]]) -> Box<[TreeFile]> {
    let mut tree = List::with_capacity(u32::try_from(paths.len()).expect("fixture count"));
    for path in paths {
        tree.push(TreeFile { path: copy_of(path), source: Source::Static { sha256: Digest([1; 32]) }, mode: 0o644 })
            .expect("fixture count");
    }
    tree.into_boxed()
}
pub fn file<const N: usize>(key: &[u8], fields: [Made; N]) -> Concrete {
    Concrete { key: copy_of(key), at: FILE_AT, element: None, fields: Box::new(fields) }
}
pub fn text(name: Name, value: &[u8]) -> Made {
    Made { name, at: FIELD_AT, value: Fielded::Text(copy_of(value)) }
}
pub fn directory(value: bool) -> Made {
    Made { name: Name::Directory, at: FIELD_AT, value: Fielded::Bool(value) }
}
pub fn plan(tree: &[TreeFile], files: &[Concrete], dropped: &[&[u8]]) -> Result<Plan, Box<[Fault]>> {
    let mut rows = List::with_capacity(u32::try_from(files.len()).expect("fixture count"));
    for entry in files {
        rows.push(Located { task: b"motd", entry }).expect("fixture count");
    }
    let mut paths = List::with_capacity(u32::try_from(dropped.len()).expect("fixture count"));
    for path in dropped {
        paths.push(copy_of(path)).expect("fixture count");
    }
    let mut faults = List::with_capacity(crate::limits::SHIPPED.expansion);
    let plan = files::plan(b"motd", tree, rows.as_slice(), paths.as_slice(), &mut faults);
    if faults.is_empty() { Ok(plan) } else { Err(faults.into_boxed()) }
}

pub type Owners<'a> = (&'a [u8], &'a [u8], &'a [u8]);
pub fn owners(plan: &Plan) -> Box<[Owners<'_>]> {
    let mut entries = List::with_capacity(u32::try_from(plan.installs.len()).expect("plan count"));
    for install in &plan.installs {
        entries.push((install.path.as_ref(), install.owner.as_ref(), install.group.as_ref())).expect("plan count");
    }
    entries.into_boxed()
}
pub fn template_plans(entries: &[crate::revision::Entry]) -> Box<[Plan]> {
    let mut tree = List::with_capacity(u32::try_from(entries.len()).expect("fixture count"));
    for entry in entries {
        let crate::revision::Class::Tree { path, .. } = crate::revision::class(&entry.path) else {
            continue;
        };
        let path = path.strip_suffix(b".j2").expect("template suffix");
        let mut absolute = skein_lib::Writer::new(path.len().saturating_add(1));
        absolute.put(b"/").expect("prefix");
        absolute.put(path).expect("path");
        tree.push(TreeFile {
            path: absolute.finish(),
            source: Source::Template { oid: entry.oid.clone(), path: entry.path.clone() },
            mode: 0o644,
        })
        .expect("fixture count");
    }
    Box::new([files::plan(b"motd", tree.as_slice(), &[], &[], &mut List::with_capacity(0))])
}

mod sources;
pub use sources::*;

mod accounts;
pub use accounts::*;

pub mod data;

mod directives;
pub use directives::*;
