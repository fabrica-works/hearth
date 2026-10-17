//! Download declarations (format.md, 10.1).
use super::secrets::{fault, formed, mode_with_default, required as required_text, text};
use crate::{
    entries::Located,
    errors::{Error, Format, Schema},
    parts::software::{Download, Fetch, How},
    revision::declaration::Name,
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::bytes::copy_of;
pub(crate) fn entry(located: &Located<'_>) -> Result<(Download, Fetch), Box<Error>> {
    match check(located) {
        Ok(row) => Ok(row),
        Err((name, schema)) => Err(Box::new(fault(located, Some(name), schema))),
    }
}
fn check(located: &Located<'_>) -> Result<(Download, Fetch), (Name, Schema)> {
    let entry = located.entry;
    let url = required(entry, Name::Url)?;
    let sha = required(entry, Name::Sha256)?;
    checked(sha, Format::Sha256, Name::Sha256)?;
    let mut bytes = [0; 32];
    for (byte, pair) in bytes.iter_mut().zip(sha.chunks_exact(2)) {
        let mut value = 0_u8;
        for digit in pair {
            value = value.saturating_mul(16).saturating_add(if digit.is_ascii_digit() {
                digit.saturating_sub(b'0')
            } else {
                digit.saturating_sub(b'a').saturating_add(10)
            });
        }
        *byte = value;
    }
    let sha256 = Digest(bytes);
    let path = required(entry, Name::Path)?;
    checked(path, Format::Path, Name::Path)?;
    if !(path.starts_with(b"/opt/") || path.starts_with(b"/usr/local/")) {
        return Err((Name::Path, Schema::Beneath { allowed: Box::new([copy_of(b"/opt"), copy_of(b"/usr/local")]) }));
    }
    let unpack = required(entry, Name::Unpack)?;
    let how = match unpack {
        b"file" => {
            absent(entry, Name::Strip, Name::Mode)?;
            let mode = match mode_with_default(entry, b"0755") {
                Ok(mode) => mode,
                Err(schema) => return Err((Name::Mode, schema)),
            };
            if mode > 0o777 {
                return Err((Name::Mode, Schema::Above { max: 0o777 }));
            }
            How::File { mode }
        }
        b"tar" => {
            absent(entry, Name::Mode, Name::Strip)?;
            let strip = entry.int(Name::Strip).unwrap_or(0);
            if strip < 0 {
                return Err((Name::Strip, Schema::Below { min: 0 }));
            }
            if strip > 255 {
                return Err((Name::Strip, Schema::Above { max: 255 }));
            }
            How::Tar { strip: u32::try_from(strip).expect("checked strip") }
        }
        value => {
            return Err((
                Name::Unpack,
                Schema::Enum { value: copy_of(value), listed: Box::new([copy_of(b"file"), copy_of(b"tar")]) },
            ));
        }
    };
    Ok((
        Download {
            path: copy_of(path),
            sha256,
            owner: text(entry, Name::Owner, b"root"),
            group: text(entry, Name::Group, b"root"),
            how,
        },
        Fetch { sha256, url: copy_of(url) },
    ))
}
fn absent(entry: &crate::entries::Concrete, name: Name, allowed: Name) -> Result<(), (Name, Schema)> {
    for field in &entry.fields {
        if field.name == name {
            return Err((
                name,
                Schema::UndeclaredField {
                    field: copy_of(name.key()),
                    declared: Box::new([
                        copy_of(b"url"),
                        copy_of(b"sha256"),
                        copy_of(b"unpack"),
                        copy_of(b"path"),
                        copy_of(b"owner"),
                        copy_of(b"group"),
                        copy_of(allowed.key()),
                    ]),
                },
            ));
        }
    }
    Ok(())
}

fn required(entry: &crate::entries::Concrete, name: Name) -> Result<&[u8], (Name, Schema)> {
    match required_text(entry, name) {
        Ok(value) => Ok(value),
        Err(schema) => Err((name, schema)),
    }
}
fn checked(value: &[u8], format: Format, name: Name) -> Result<(), (Name, Schema)> {
    match formed(value, format) {
        Ok(()) => Ok(()),
        Err(schema) => Err((name, schema)),
    }
}
