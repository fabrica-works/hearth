use super::{FIELD_AT, file};
use crate::{
    entries::{Concrete, Expanded, Expansion, Fielded, Group, Made},
    errors::Error,
    kinds::users::Accounts,
    parts::config::Config,
    revision::declaration::{Map, Name},
};
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
pub const MIN_UID: i64 = 1;
pub const MAX_UID: i64 = 65533;
pub const MIN_SUBORDINATE: i64 = 65536;
pub const MAX_SUBORDINATE_COUNT: i64 = 4_294_901_759;
pub fn integer(name: Name, value: i64) -> Made {
    Made { name, at: FIELD_AT, value: Fielded::Int(value) }
}
pub fn range(base: i64, count: i64) -> Made {
    Made { name: Name::Subordinate, at: FIELD_AT, value: Fielded::Range { base, count } }
}
pub fn strings(name: Name, values: &[&[u8]]) -> Made {
    let mut strings = List::with_capacity(u32::try_from(values.len()).expect("fixture count"));
    for value in values {
        strings.push(copy_of(value)).expect("fixture count");
    }
    Made { name, at: FIELD_AT, value: Fielded::Strings(strings.into_boxed()) }
}
pub fn account_expansion(tasks: &[(&[u8], Map, &[Concrete])]) -> Expansion {
    let mut expanded = List::with_capacity(u32::try_from(tasks.len()).expect("fixture count"));
    for (task, map, entries) in tasks {
        let mut concrete = List::with_capacity(u32::try_from(entries.len()).expect("fixture count"));
        for entry in *entries {
            concrete.push(entry.clone()).expect("fixture count");
        }
        expanded
            .push(Expanded {
                task: copy_of(task),
                groups: Box::new([Group { map: *map, entries: concrete.into_boxed() }]),
            })
            .expect("fixture count");
    }
    Expansion { tasks: expanded.into_boxed() }
}
pub fn accounts<const N: usize>(fields: [Made; N]) -> Result<Accounts, Box<[Error]>> {
    crate::parts::config::accounts(&account_expansion(&[(b"motd", Map::Users, &[file(b"free", fields)])]))
}
pub fn account_config(expansion: &Expansion) -> Config {
    crate::parts::config::account_rows(
        &crate::parts::config::accounts(expansion).expect("valid accounts"),
        Box::new([]),
    )
}

pub const PACKAGE_LIMITS: crate::kinds::users::UserKind =
    crate::kinds::users::UserKind::Package { uid: Some(1), gid: Some(65533) };
pub const SUBORDINATE_LIMITS: (u32, u32) = (65536, 4_294_901_759);
