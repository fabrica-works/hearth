//! The walk's bounds (domain/store.md, 5.6) and its worst case.
use crate::{
    Level,
    links::Link,
    vocabulary::{Item, Xattr},
};
use alloc::boxed::Box;
use skein_lib::{List, Map, Stack};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// Entries in a layer.
    pub entries: u32,
    /// Names in one directory.
    pub names: u32,
    /// Directories nested below the root.
    pub depth: u32,
    /// Names held across the stack, in all.
    pub held: u32,
    /// A name: `NAME_MAX`.
    pub name: u32,
    /// A path.
    pub path: u32,
    /// A link's target.
    pub target: u32,
    /// Covered attributes of one entry.
    pub attributes: u32,
    /// An attribute's name: `XATTR_NAME_MAX`.
    pub attribute_name: u32,
    /// One attribute's value: `XATTR_SIZE_MAX`.
    pub attribute_value: u32,
    /// Inodes with more than one link.
    pub linked: u32,
    /// Their first paths, in all.
    pub linked_paths: u32,
    /// Names in a page of a directory's listing.
    pub page: u32,
    /// Names in an attribute list, covered or not.
    pub listed: u32,
    /// Bytes of items in one `Write`, as `batch::weight` estimates them.
    pub batch: u32,
}
pub const SHIPPED: Limits = Limits {
    entries: 2_000_000,
    names: 262_144,
    depth: 256,
    held: 1_048_576,
    name: 255,
    path: 4_096,
    target: 4_096,
    attributes: 64,
    attribute_name: 255,
    attribute_value: 65_536,
    linked: 65_536,
    linked_paths: 16_777_216,
    page: 1_024,
    listed: 1_024,
    batch: 262_144,
};

/// The most items a batch of `limits.batch` bytes holds: each weighs at least an `Item`, and an
/// entry with its put, whatever they weigh, goes alone.
pub(crate) fn batch_items(limits: &Limits) -> u32 {
    let least = u32::try_from(size_of::<Item>()).expect("an item is smaller than a u32");
    limits.batch.checked_div(least).expect("an item has a size").max(2)
}

/// The longest path a walk names: a directory's, a `/` and a name.
fn path(limits: &Limits) -> Option<u64> {
    u64::from(limits.path).checked_add(1)?.checked_add(u64::from(limits.name))
}

/// The heaviest entry with its put, as `batch::weight` counts them.
fn pair(limits: &Limits) -> Option<u64> {
    let item = u64::try_from(size_of::<Item>()).ok()?;
    let xattr = u64::try_from(size_of::<Xattr>()).ok()?;
    let attribute =
        u64::from(limits.attribute_name).checked_add(u64::from(limits.attribute_value))?.checked_add(xattr)?;
    let attributes = attribute.checked_mul(u64::from(limits.attributes))?;
    let payload = u64::from(limits.target).max(path(limits)?);
    item.checked_mul(2)?.checked_add(path(limits)?.checked_mul(2)?)?.checked_add(attributes)?.checked_add(payload)
}

/// The walk's state and the events in flight: the names of each directory on the stack, bounded
/// in all; the entry in hand and its attributes; the linked inodes; the batch, twice, as a
/// flushed one is in the step that starts the next; and a page.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    let levels = limits.depth.checked_add(1)?;
    let paths = path(limits)?;
    let maps = u64::from(levels.checked_add(2)?);
    let name = u64::from(limits.name);
    let names = u64::from(limits.held).checked_add(u64::from(limits.page.max(limits.listed)))?;
    let stack = Stack::<Level>::worst_case(levels)?
        .checked_add(paths.checked_mul(u64::from(levels))?)?
        .checked_add(Map::<Box<[u8]>, ()>::worst_case(limits.held)?)?
        .checked_add(Map::<Box<[u8]>, ()>::worst_case(0)?.checked_mul(maps)?)?
        .checked_add(names.checked_mul(name)?)?
        .checked_add(
            u64::from(limits.page.max(limits.listed)).checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?,
        )?;
    let attributes = Map::<Box<[u8]>, ()>::worst_case(limits.attributes)?
        .checked_add(List::<Xattr>::worst_case(limits.attributes)?.checked_mul(2)?)?
        .checked_add(u64::from(limits.attributes).checked_mul(
            name.checked_add(u64::from(limits.attribute_name))?.checked_add(u64::from(limits.attribute_value))?,
        )?)?;
    let linked = Map::<u64, Link>::worst_case(limits.linked)?.checked_add(u64::from(limits.linked_paths))?;
    let items = List::<Item>::worst_case(batch_items(limits))?;
    let batch = items.checked_add(u64::from(limits.batch).max(pair(limits)?))?.checked_mul(2)?;
    // The entry in hand, its request's path, its item's two and its failure's; and a hard link's
    // digest, whose encoder holds its nesting and a key.
    let encoder = u64::from(hearth_lib::encode::DEPTH).checked_mul(64)?.checked_add(name)?.checked_add(256)?;
    let entry = paths.checked_mul(6)?.checked_add(u64::from(limits.target))?.checked_add(encoder)?;
    stack
        .checked_add(attributes)?
        .checked_add(linked)?
        .checked_add(batch)?
        .checked_add(entry)?
        .checked_add(u64::try_from(size_of::<crate::Tree>()).ok()?)
}
