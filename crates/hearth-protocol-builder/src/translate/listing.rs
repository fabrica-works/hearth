//! Listed entries retain every field (domain/render.md, 2.2).
use crate::{git_machine::listing, render::revision};
use skein_lib::bytes;
#[must_use]
pub fn entry(entry: listing::Entry) -> revision::Entry {
    let listing::Entry { path, mode, oid, size } = entry;
    revision::Entry { path, mode: mode_to_render(mode), oid: revision::Oid(bytes::copy_of(oid.hex())), size }
}
#[must_use]
pub fn mode_to_render(mode: listing::Mode) -> revision::Mode {
    match mode {
        listing::Mode::File => revision::Mode::File,
        listing::Mode::Executable => revision::Mode::Executable,
        listing::Mode::Link => revision::Mode::Link,
        listing::Mode::Submodule => revision::Mode::Submodule,
    }
}
