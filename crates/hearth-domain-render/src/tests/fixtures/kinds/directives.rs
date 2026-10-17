use super::{file, text};
use crate::{
    entries::Located,
    errors::Error,
    parts::config::{Preset, Tmpfile},
    revision::declaration::Name,
};
pub fn preset<const N: usize>(fields: [crate::entries::Made; N]) -> Result<Preset, Box<Error>> {
    let entry = file(b"ssh.service", fields);
    crate::kinds::presets::entry(&Located { task: b"motd", entry: &entry })
}
pub fn tmpfile(kind: Option<&[u8]>, mode: Option<&[u8]>) -> Result<Tmpfile, Box<Error>> {
    let mut fields = skein_lib::List::with_capacity(2);
    if let Some(kind) = kind {
        fields.push(text(Name::Type, kind)).expect("room");
    }
    if let Some(mode) = mode {
        fields.push(text(Name::Mode, mode)).expect("room");
    }
    let mut entry = file(b"/var/log/journal", []);
    entry.fields = fields.into_boxed();
    crate::kinds::tmpfiles::entry(&Located { task: b"motd", entry: &entry })
}
