//! Declared script files (protocol/format.md, 5.8); software content belongs to its pass.
use crate::{
    errors::{Error, Kind, Place, Repository},
    revision::{self, declaration::Declaration},
    vocabulary::Class,
};
use alloc::boxed::Box;
use skein_lib::{List, Writer, bytes::copy_of};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Source {
    pub task: Box<[u8]>,
    pub path: Box<[u8]>,
    pub order: u16,
}
pub(crate) fn plan(
    task: &[u8],
    declaration: &Declaration,
    listing: &[revision::Entry],
    faults: &mut List<Error>,
) -> Box<[Source]> {
    let mut sources = List::with_capacity(u32::try_from(declaration.scripts.len()).expect("declaration bound"));
    for script in &declaration.scripts {
        let mut source = None;
        for entry in listing {
            if let revision::Class::Script { task: name, name: file } = revision::class(&entry.path)
                && name == task
                && file.strip_suffix(b".j2").unwrap_or(file) == script.fragment.name()
            {
                source = Some(entry.path.clone());
            }
        }
        if let Some(path) = source {
            sources.push(Source { task: copy_of(task), path, order: script.order }).expect("declaration bound");
        } else {
            let mut path = Writer::new(task.len().saturating_add(16));
            path.put(b"tasks/").expect("path room");
            path.put(task).expect("path room");
            path.put(b"/task.json").expect("path room");
            faults
                .push(error(task, Place { path: path.finish(), position: Some(script.at) }, Repository::NoScript))
                .expect("fault room");
        }
    }
    for entry in listing {
        let revision::Class::Script { task: name, name: file } = revision::class(&entry.path) else { continue };
        if name != task {
            continue;
        }
        let mut declared = false;
        for script in &declaration.scripts {
            declared |= file.strip_suffix(b".j2").unwrap_or(file) == script.fragment.name();
        }
        if !declared {
            faults
                .push(error(task, Place { path: entry.path.clone(), position: None }, Repository::Stray))
                .expect("fault room");
        }
    }
    sources.into_boxed()
}
fn error(task: &[u8], at: Place, repository: Repository) -> Error {
    Error {
        side: None,
        kind: Kind::Repository(repository),
        class: Class::Configuration,
        at: Some(at),
        also: None,
        target: None,
        task: Some(copy_of(task)),
    }
}
