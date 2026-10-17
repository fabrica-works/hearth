use super::{kinds, packages};
use crate::{
    kinds::users::Accounts,
    parts::packages::{Initramfs, Microcode},
};
use skein_lib::{List, bytes::copy_of};
pub fn accounts_with_ranges(rows: &[(&[u8], i64, i64)]) -> Accounts {
    let mut users = List::with_capacity(u32::try_from(rows.len()).expect("fixture count"));
    for &(name, base, count) in rows {
        let accounts = kinds::accounts([kinds::range(base, count)]).expect("valid range");
        let mut user = super::only(&accounts.users).clone();
        user.name = copy_of(name);
        users.push(user).expect("fixture count");
    }
    Accounts { users: users.into_boxed(), groups: Box::new([]) }
}
pub fn initramfs(microcode: Microcode, modules: &[&[u8]]) -> Initramfs {
    let mut names = List::with_capacity(u32::try_from(modules.len()).expect("fixture count"));
    for module in modules {
        names.push(copy_of(module)).expect("fixture count");
    }
    Initramfs { microcode, modules: names.into_boxed() }
}
pub fn packages_with_file(file: Option<hearth_lib::encode::Digest>) -> crate::parts::packages::Packages {
    let mut packages = packages::thin_packages();
    packages.initramfs_file = file;
    packages
}
