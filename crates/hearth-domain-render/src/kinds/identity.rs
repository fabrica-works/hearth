//! M1 identity is an SSH host key (format.md, 9.3).
use super::secrets::{fault, formed, mode, required, text};
use crate::{
    entries::Located,
    errors::{Error, Format, Schema},
    parts::secrets::{Algorithm, Identity, IdentityKind},
    revision::declaration::Name,
};
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
const KINDS: &[&[u8]] = &[b"ssh-host-key"];
const ALGORITHMS: &[&[u8]] = &[b"ed25519", b"ecdsa", b"rsa"];
fn enumeration(value: &[u8], allowed: &[&[u8]]) -> Schema {
    let mut values = skein_lib::List::with_capacity(u32::try_from(allowed.len()).expect("enum count"));
    for value in allowed {
        values.push(copy_of(value)).expect("enum count");
    }
    Schema::Enum { value: copy_of(value), listed: values.into_boxed() }
}
pub(crate) fn entry(located: &Located<'_>) -> Result<Identity, Box<Error>> {
    let entry = located.entry;
    for name in [Name::Kind, Name::Algorithm, Name::Path] {
        if let Err(schema) = required(entry, name) {
            return Err(Box::new(fault(located, None, schema)));
        }
    }
    if entry.text(Name::Kind) != Some(b"ssh-host-key") {
        return Err(Box::new(fault(
            located,
            Some(Name::Kind),
            enumeration(entry.text(Name::Kind).expect("required kind"), KINDS),
        )));
    }
    let algorithm = match entry.text(Name::Algorithm).expect("required algorithm") {
        b"ed25519" => Algorithm::Ed25519,
        b"ecdsa" => Algorithm::Ecdsa,
        b"rsa" => Algorithm::Rsa,
        _bytes => {
            return Err(Box::new(fault(
                located,
                Some(Name::Algorithm),
                enumeration(entry.text(Name::Algorithm).expect("required algorithm"), ALGORITHMS),
            )));
        }
    };
    for name in [Name::Path, Name::PublicPath] {
        if let Some(path) = entry.text(name)
            && let Err(schema) = formed(path, Format::Path)
        {
            return Err(Box::new(fault(located, Some(name), schema)));
        }
    }
    let mode = match mode(entry) {
        Ok(mode) => mode,
        Err(schema) => return Err(Box::new(fault(located, Some(Name::Mode), schema))),
    };
    let mut public_path = None;
    if let Some(path) = entry.text(Name::PublicPath) {
        public_path = Some(copy_of(path));
    }
    Ok(Identity {
        name: entry.key.clone(),
        kind: IdentityKind::SshHostKey { algorithm },
        path: text(entry, Name::Path, b""),
        public_path,
        mode,
        owner: text(entry, Name::Owner, b"root"),
        group: text(entry, Name::Group, b"root"),
    })
}
