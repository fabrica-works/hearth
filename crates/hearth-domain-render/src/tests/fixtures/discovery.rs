use super::{bytes, composition};

/// Typed documents for a base and its optional member patches.
pub fn member_documents(
    base: Option<crate::revision::Members>,
    host: Option<crate::revision::Members>,
) -> crate::revision::Documents {
    use crate::revision::{Document, Entry, Layer, Mode, Oid};
    let mut entries = skein_lib::List::with_capacity(3);
    for path in [b"envs/lab.json".as_slice(), b"hosts/stoa/stoa-lab.json", b"hosts/stoa/stoa.json"] {
        entries
            .push(Entry { path: bytes::copy_of(path), mode: Mode::File, oid: Oid(bytes::copy_of(path)), size: 2 })
            .expect("three documents");
    }
    let mut documents = crate::revision::Documents::new(entries.as_slice());
    for (path, members) in
        [(b"envs/lab.json".as_slice(), None), (b"hosts/stoa/stoa-lab.json", host), (b"hosts/stoa/stoa.json", base)]
    {
        documents.set(
            &Oid(bytes::copy_of(path)),
            Document { at: composition::at(1, 1), layer: Layer { placement: None, tasks: Box::new([]) }, members },
        );
    }
    documents
}
pub fn members(names: &[(&[u8], bool)]) -> crate::revision::Members {
    let mut each = skein_lib::List::with_capacity(u32::try_from(names.len()).expect("fixture members"));
    for &(name, removed) in names {
        let layer = if removed { None } else { Some(crate::revision::Layer { placement: None, tasks: Box::new([]) }) };
        each.push(crate::revision::Member { name: bytes::copy_of(name), at: composition::at(1, 12), layer })
            .expect("fixture members");
    }
    crate::revision::Members { at: composition::at(1, 11), each: each.into_boxed() }
}
pub fn with_document(
    documents: &crate::revision::Documents,
    path: &[u8],
    document: crate::revision::Document,
) -> crate::revision::Documents {
    let mut entries = skein_lib::List::with_capacity(
        u32::try_from(documents.entries().len().saturating_add(1)).expect("fixture count"),
    );
    for entry in documents.entries() {
        entries.push(entry.clone()).expect("fixture count");
    }
    let oid = crate::revision::Oid(bytes::copy_of(path));
    entries
        .push(crate::revision::Entry {
            path: bytes::copy_of(path),
            mode: crate::revision::Mode::File,
            oid: oid.clone(),
            size: 2,
        })
        .expect("fixture count");
    let mut extended = crate::revision::Documents::new(entries.as_slice());
    for entry in documents.entries() {
        if let Some(index) = documents.find(&entry.path)
            && let Some(document) = documents.get(index)
        {
            extended.set(&entry.oid, document.clone());
        }
    }
    extended.set(&oid, document);
    extended
}
pub fn member_layers(values: [Option<i64>; 5]) -> crate::revision::Documents {
    use crate::revision::{Oid, Task};
    let [base, member, environment, host, host_member] = values;
    let name = (b"1".as_slice(), false);
    let mut documents =
        member_documents(Some(members(core::slice::from_ref(&name))), Some(members(core::slice::from_ref(&name))));
    for (path, value, member) in [
        (b"hosts/stoa/stoa.json".as_slice(), base, false),
        (b"hosts/stoa/stoa.json", member, true),
        (b"envs/lab.json", environment, false),
        (b"hosts/stoa/stoa-lab.json", host, false),
        (b"hosts/stoa/stoa-lab.json", host_member, true),
    ] {
        let index = documents.find(path).expect("fixture path");
        let mut document = documents.get(index).expect("fixture document").clone();
        let mut tasks = skein_lib::List::with_capacity(1);
        if let Some(value) = value {
            tasks
                .push(Task {
                    name: bytes::copy_of(b"motd"),
                    at: composition::at(1, 2),
                    value: composition::map(
                        composition::at(1, 10),
                        &[(b"k", composition::at(1, 11), composition::int(composition::at(1, 15), value))],
                    ),
                })
                .expect("one task");
        }
        if member {
            for each in &mut document.members.as_mut().expect("members").each {
                each.layer.as_mut().expect("present member").tasks = tasks.as_slice().into();
            }
        } else {
            document.layer.tasks = tasks.into_boxed();
        }
        documents.set(&Oid(bytes::copy_of(path)), document);
    }
    documents
}
