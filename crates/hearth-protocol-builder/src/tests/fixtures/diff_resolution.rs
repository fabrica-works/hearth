use crate::{records::diff as source, render::diff as target};
use skein_lib::List;
#[must_use]
pub fn resolution_change_back(change: source::ResolutionChange) -> target::ResolutionChange {
    match change {
        source::ResolutionChange::Same => target::ResolutionChange::Same,
        source::ResolutionChange::NewKey => target::ResolutionChange::NewKey,
        source::ResolutionChange::Resolved { added, removed, versions_changed } => {
            let mut moved = List::with_capacity(u32::try_from(versions_changed.len()).expect("bounded closure"));
            for source::Moved { name, architecture, before, after } in versions_changed {
                moved.push(target::Moved { name, architecture, before, after }).expect("closure capacity");
            }
            target::ResolutionChange::Resolved {
                added: listed(added),
                removed: listed(removed),
                versions_changed: moved.into_boxed(),
            }
        }
    }
}
fn listed(packages: Box<[source::Listed]>) -> Box<[target::Listed]> {
    let mut translated = List::with_capacity(u32::try_from(packages.len()).expect("bounded closure"));
    for source::Listed { name, architecture, version } in packages {
        translated.push(target::Listed { name, architecture, version }).expect("closure capacity");
    }
    translated.into_boxed()
}

#[must_use]
pub fn resolution_changes() -> [target::ResolutionChange; 3] {
    [
        target::ResolutionChange::Same,
        target::ResolutionChange::NewKey,
        target::ResolutionChange::Resolved {
            added: Box::new([target::Listed {
                name: b"vim".as_slice().into(),
                architecture: b"amd64".as_slice().into(),
                version: b"9".as_slice().into(),
            }]),
            removed: Box::new([target::Listed {
                name: b"nano".as_slice().into(),
                architecture: b"all".as_slice().into(),
                version: b"8".as_slice().into(),
            }]),
            versions_changed: Box::new([target::Moved {
                name: b"apt".as_slice().into(),
                architecture: b"amd64".as_slice().into(),
                before: b"2".as_slice().into(),
                after: b"3".as_slice().into(),
            }]),
        },
    ]
}
