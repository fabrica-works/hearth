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
