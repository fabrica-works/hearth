use crate::{
    entries::Expansion,
    errors::{Reserved, Rule},
    reserved::{self, Refused},
    revision::declaration::{Map, Name},
};
pub const COMMAND_LINE_LIMIT: usize = 2048; // machine.md, 6.2: COMMAND_LINE_SIZE.
pub const X1_EFI_BYTES: usize = 139; // 118 + 3 * x1-prod's seven bytes, session 21 Choices 6.
pub const FITTING_PARAMETER: usize = COMMAND_LINE_LIMIT - X1_EFI_BYTES - 1; // One separating space.
pub const PARAM_BYTES: &[u8; FITTING_PARAMETER + 1] = &[b'x'; FITTING_PARAMETER + 1];
pub const LONG_PARAMETER: usize = 1000; // Two such parameters make WAIT's 2141-byte line.
pub fn refused(map: Map, key: &[u8], fields: &[(Name, &[u8])], container: bool) -> Box<[Refused]> {
    check(&mut super::secret_entry(map, key, fields), container)
}
pub fn check(expansion: &mut Expansion, container: bool) -> Box<[Refused]> {
    reserved::entries(expansion, b"x1-prod", container)
}
pub fn reserved(faults: &[Refused]) -> Box<[Reserved]> {
    let mut reserved = skein_lib::List::with_capacity(u32::try_from(faults.len()).expect("fault count"));
    for fault in faults {
        reserved.push(fault.reserved.clone()).expect("fault count");
    }
    reserved.into_boxed()
}
pub fn rules(faults: &[Refused]) -> Box<[Rule]> {
    let mut rules = skein_lib::List::with_capacity(u32::try_from(faults.len()).expect("fault count"));
    for fault in faults {
        rules.push(fault.reserved.rule).expect("fault count");
    }
    rules.into_boxed()
}
pub fn command(param: &[u8]) -> Expansion {
    super::boot_expansion(&[(b"base", b"key", param, Some(i64::from(super::DEFAULT_CMDLINE_ORDER)))])
}
pub fn word_rules(map: Map, key: &[u8], fields: &[(Name, &[u8])], container: bool) -> Box<[Rule]> {
    rules(&refused(map, key, fields, container))
}
pub fn installed<'a>(
    task: &'a [u8],
    path: &'a [u8],
    at: Option<crate::revision::Position>,
    shape: reserved::Shape,
) -> reserved::Installed<'a> {
    reserved::Installed { task, path, at, shape }
}
pub fn tree(path: &[u8], shape: reserved::Shape, at: Option<crate::revision::Position>) -> Box<[Refused]> {
    reserved::tree(core::slice::from_ref(&installed(b"motd", path, at, shape)))
}
pub fn tree_rules(path: &[u8], shape: reserved::Shape) -> Box<[Rule]> {
    rules(&tree(path, shape, None))
}
pub type EntryPathCase<'a> = (Map, &'a [u8], &'a [u8], &'a [Rule]);
pub type DataPathCase<'a> = (&'a [u8], &'a [u8], &'a [u8], &'a [Rule]);
pub type TmpfilePathCase<'a> = (&'a [u8], &'a [(Name, &'a [u8])], &'a [Rule]);
pub fn data_rules(key: &[u8], class: &[u8], path: &[u8]) -> Box<[Rule]> {
    word_rules(Map::Data, key, &[(Name::Class, class), (Name::Path, path)], false)
}
pub fn live(
    action: &[u8],
    unit: Option<&[u8]>,
    paths: &[&[u8]],
    installed: &[reserved::Installed<'_>],
) -> Box<[Refused]> {
    reserved::live(&super::live_entry(action, unit, paths), installed)
}
pub fn live_rules(action: &[u8], unit: Option<&[u8]>, paths: &[&[u8]]) -> Box<[Rule]> {
    rules(&live(action, unit, paths, &[]))
}
pub fn hearth_groups(group: &[u8], groups: &[&[u8]]) -> Expansion {
    hearth_with_secret(
        group,
        groups,
        super::kinds::file(b"mail/tls/mx1.key", [super::kinds::text(Name::Group, b"stewards")]),
    )
}
pub fn default_hearth_group(group: &[u8]) -> Expansion {
    hearth_with_secret(group, &[], super::kinds::file(b"mail/tls/mx1.key", []))
}
fn hearth_with_secret(group: &[u8], groups: &[&[u8]], secret: crate::entries::Concrete) -> Expansion {
    use super::kinds::{account_expansion, file, strings, text};
    account_expansion(&[
        (b"hearth", Map::Users, &[file(b"hearth", [text(Name::Group, group), strings(Name::Groups, groups)])]),
        (b"mail", Map::Secrets, core::slice::from_ref(&secret)),
    ])
}

pub const HEARTH_TASKS: &[(&[u8], &[Rule])] = &[(b"hearth", &[Rule::NeverLivePath]), (b"other", &[])];
pub fn live_with_download(task: &[u8]) -> Expansion {
    use super::kinds::{account_expansion, file, strings, text};
    account_expansion(&[
        (task, Map::Software, &[file(b"tool", [text(Name::Path, TOOL_PATH)])]),
        (b"mail", Map::Live, &[file(b"tool", [text(Name::Action, b"none"), strings(Name::Paths, &[TOOL_BINARY])])]),
    ])
}
pub type EntryFieldCase<'a> = (&'a [(Name, &'a [u8])], &'a [Rule]);
pub fn ordered_secret(fields: &[(Name, &[u8])]) -> Expansion {
    let mut expansion = super::secret_entry(Map::Secrets, b"key", fields);
    for task in &mut expansion.tasks {
        for group in &mut task.groups {
            for entry in &mut group.entries {
                for (index, field) in entry.fields.iter_mut().enumerate() {
                    field.at.column =
                        super::FIELD_AT.column.saturating_add(u32::try_from(index).expect("fixture field count"));
                }
            }
        }
    }
    expansion
}

pub type LiveUnitCase<'a> = (Option<&'a [u8]>, &'a [u8], &'a [Rule]);
pub type HearthGroupCase<'a> = (&'a [u8], &'a [&'a [u8]], &'a [Rule]);

pub const TOOL_PATH: &[u8] = b"/opt/tool";
pub const TOOL_BINARY: &[u8] = b"/opt/tool/bin/tool";
pub fn live_unit_rules(action: &[u8], unit: Option<&[u8]>) -> Box<[Rule]> {
    live_rules(action, unit, &[TOOL_PATH])
}
