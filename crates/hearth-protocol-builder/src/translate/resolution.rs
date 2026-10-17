//! Resolution inputs and apt's install triples (render.md, 5.2 and 5.5).
use crate::render::{parts::packages as target, resolution};
use alloc::boxed::Box;
use hearth_protocol_mmdebstrap::{limits::Limits, output, tail, vector};
use skein_lib::{List, bytes};
#[must_use]
pub fn simulate(
    sources: &target::Sources,
    names: &[Box<[u8]>],
    variant: target::Variant,
    apt: &target::Apt,
    limits: &Limits,
) -> Option<vector::Simulate> {
    let variant = match variant {
        target::Variant::Apt => vector::Variant::Apt,
        target::Variant::Container => return None,
    };
    let mut request = List::with_capacity(limits.request);
    for name in names {
        request.push(vector::Name::new(name, limits)?).ok()?;
    }
    let mut components = List::with_capacity(4);
    for component in &sources.components {
        components
            .push(match component {
                target::Component::Main => vector::Component::Main,
                target::Component::Contrib => vector::Component::Contrib,
                target::Component::NonFree => vector::Component::NonFree,
                target::Component::NonFreeFirmware => vector::Component::NonFreeFirmware,
            })
            .ok()?;
    }
    Some(vector::Simulate {
        sources: vector::Sources {
            suite: vector::Name::new(&sources.suite, limits)?,
            architecture: vector::Name::new(&sources.architecture, limits)?,
            components: components.into_boxed(),
            archive: vector::Timestamp::new(&sources.archive)?,
            updates: sources.updates,
            security: match &sources.security {
                Some(time) => Some(vector::Timestamp::new(time)?),
                None => None,
            },
        },
        request: request.into_boxed(),
        variant,
        apt: vector::Apt { install_recommends: apt.install_recommends },
    })
}
#[must_use]
pub fn inst(inst: output::Inst) -> resolution::Inst {
    resolution::Inst {
        name: bytes::copy_of(inst.name.as_bytes()),
        version: inst.version,
        architecture: bytes::copy_of(inst.architecture.as_bytes()),
    }
}
#[must_use]
pub fn cause(cause: tail::Cause, lines: &[vector::Line], tail: &[Box<[u8]>]) -> resolution::Cause {
    match cause {
        tail::Cause::Fetch => resolution::Cause::Fetch,
        tail::Cause::Resolve => resolution::Cause::Resolve,
        tail::Cause::Space => resolution::Cause::Space,
        tail::Cause::Unknown(name) => resolution::Cause::Unknown(bytes::copy_of(name.as_bytes())),
        tail::Cause::Unsatisfiable(name) => resolution::Cause::Unsatisfiable(optional_name(name)),
        tail::Cause::Other => resolution::Cause::Other,
        tail::Cause::Absent => {
            let line = absent_line(lines, tail);
            resolution::Cause::Absent { uri: line.uri.clone(), suite: line.suite.clone() }
        }
    }
}

fn absent_line<'a>(lines: &'a [vector::Line], tail: &[Box<[u8]>]) -> &'a vector::Line {
    for line in lines {
        for text in tail {
            if (text.starts_with(b"Err:") || bytes::find(text, b"does not have a Release file").is_some())
                && bytes::find(text, &line.uri).is_some()
            {
                return line;
            }
        }
    }
    lines.first().expect("one archive line")
}

fn optional_name(name: Option<vector::Name>) -> Option<Box<[u8]>> {
    Some(bytes::copy_of(name?.as_bytes()))
}

#[must_use]
pub fn stanza(stanza: hearth_protocol_mmdebstrap::packages::Stanza) -> resolution::Stanza {
    resolution::Stanza {
        package: bytes::copy_of(stanza.package.as_bytes()),
        version: stanza.version,
        architecture: bytes::copy_of(stanza.architecture.as_bytes()),
        filename: stanza.filename,
        size: stanza.size,
        sha256: crate::render::vocabulary::Digest(stanza.sha256),
    }
}
#[must_use]
pub fn line(line: &vector::Line, indices: Box<[resolution::Index]>) -> resolution::Line {
    let mut components = List::with_capacity(4);
    for component in &line.components {
        components
            .push(bytes::copy_of(match component {
                vector::Component::Main => b"main",
                vector::Component::Contrib => b"contrib",
                vector::Component::NonFree => b"non-free",
                vector::Component::NonFreeFirmware => b"non-free-firmware",
            }))
            .expect("bounded components");
    }
    resolution::Line { uri: line.uri.clone(), suite: line.suite.clone(), components: components.into_boxed(), indices }
}
