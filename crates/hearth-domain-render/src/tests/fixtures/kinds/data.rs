use super::{file, text};
use crate::{
    entries::Located,
    errors::Error,
    kinds::data::Entry,
    parts::data::{Data, DataClass, DataEntry, DataValue},
    revision::declaration::{Map, Name},
};
pub const MAX_ENTRIES: u32 = crate::parts::data::MAX_ENTRIES;
pub const MAX_NAME: usize = 32;
pub fn entry(name: &[u8], fields: &[(Name, &[u8])]) -> Result<Entry, Box<Error>> {
    let mut made = skein_lib::List::with_capacity(u32::try_from(fields.len()).expect("fields"));
    for &(name, value) in fields {
        made.push(text(name, value)).expect("fields");
    }
    let mut entry = file(name, []);
    entry.fields = made.into_boxed();
    crate::kinds::data::entry(&Located { task: b"motd", entry: &entry })
}
pub fn many(count: u32) -> crate::entries::Expansion {
    let mut expansion = crate::tests::fixtures::concrete_expansion(Map::Data, &[]);
    let mut entries = skein_lib::List::with_capacity(count);
    for number in (0..count).rev() {
        let mut name = skein_lib::Writer::new(3);
        name.put(b"x").expect("name");
        name.put(skein_lib::Decimal::of(u64::from(number) + 10).as_bytes()).expect("digits");
        let mut entry = file(&name.finish(), [text(Name::Path, b"/var/lib/app")]);
        entry.at.line = number.checked_add(1).expect("fixture line");
        entries.push(entry).expect("entries");
    }
    for task in &mut expansion.tasks {
        for group in &mut task.groups {
            group.entries = entries.clone().into_boxed();
        }
    }
    expansion
}
pub fn part(class: DataClass) -> Data {
    Data {
        entries: Box::new([DataEntry {
            name: b"iwd".as_slice().into(),
            value: DataValue { class, path: b"/var/lib/iwd".as_slice().into() },
        }]),
    }
}
