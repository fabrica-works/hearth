use crate::{
    entries::{Concrete, Expanded, Expansion, Fielded, Group, Made},
    revision::declaration::{Map, Name},
};
use skein_lib::bytes::copy_of;
type Row<'a> = (&'a [u8], &'a [(Name, &'a [u8])]);
pub fn concrete_expansion(map: Map, rows: &[Row<'_>]) -> Expansion {
    let mut entries = skein_lib::List::with_capacity(u32::try_from(rows.len()).expect("fixture count"));
    for (key, fields) in rows {
        let mut made = skein_lib::List::with_capacity(u32::try_from(fields.len()).expect("fixture count"));
        for (name, value) in *fields {
            made.push(Made { name: *name, at: super::FIELD_AT, value: Fielded::Text(copy_of(value)) })
                .expect("fixture count");
        }
        entries
            .push(Concrete {
                key: copy_of(key),
                at: super::composition::at(1, 14),
                element: None,
                fields: made.into_boxed(),
            })
            .expect("fixture count");
    }
    Expansion {
        tasks: Box::new([Expanded {
            task: copy_of(b"motd"),
            groups: Box::new([Group { map, entries: entries.into_boxed() }]),
        }]),
    }
}
pub fn secret_file(
    name: &[u8],
    path: &[u8],
    mode: u16,
    owner: &[u8],
    group: &[u8],
) -> crate::parts::secrets::SecretFile {
    crate::parts::secrets::SecretFile {
        name: copy_of(name),
        path: copy_of(path),
        mode,
        owner: copy_of(owner),
        group: copy_of(group),
    }
}
pub fn secret_entry(map: Map, key: &[u8], fields: &[(Name, &[u8])]) -> Expansion {
    concrete_expansion(map, &[(key, fields)])
}
pub fn secret_fault(map: Map, key: &[u8], fields: &[(Name, &[u8])]) -> crate::errors::Error {
    super::only(&crate::parts::secrets::build(&secret_entry(map, key, fields)).expect_err("invalid entry")).clone()
}

/// The config part remains identifiable as other parts arrive.
pub fn config_part(parts: &[crate::parts::Part]) -> &crate::parts::config::Config {
    for part in parts {
        match &part.record {
            crate::parts::PartRecord::Config(config) => return config,
            crate::parts::PartRecord::Data(_)
            | crate::parts::PartRecord::Secrets(_)
            | crate::parts::PartRecord::Software(_)
            | crate::parts::PartRecord::Live(_)
            | crate::parts::PartRecord::Packages(_)
            | crate::parts::PartRecord::Boot(_)
            | crate::parts::PartRecord::Exports(_) => {}
        }
    }
    panic!("config part")
}
