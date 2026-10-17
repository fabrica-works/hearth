//! Inputs and literal versions shared by the file-face checks.
use super::{BUILDER, Shared, THIN};
use crate::files::Files;
use hearth_fake_machine::linux::Item;
use skein_io::{digest::Digest, file};
pub const FILE_SEED: u64 = 7;
// The bound is the three bytes in the laid abc file.
pub const ABC_SIZE: u32 = 3;
pub const ABC_SHA256: Digest =
    Digest([0xba78_16bf_8f01_cfea, 0x4141_40de_5dae_2223, 0xb003_61a3_9617_7a9c, 0xb410_ff61_f200_15ad]);
pub const OTHER_SHA256: Digest = Digest([0; 4]);
pub const FILE_NAME: &[u8] = b"entry";
pub const FILE_PATH: &[u8] = b"/srv/store/cache/entry";
#[must_use]
pub fn files(laid: bool) -> Files {
    let shared = Shared::new(THIN, FILE_SEED);
    if laid {
        shared
            .borrow_mut()
            .behind
            .machine_mut()
            .linux_mut()
            .lay(&[Item::file(FILE_PATH, 0o600, b"abc").owner(BUILDER)]);
    }
    Files::new(shared, None, FILE_SEED)
}
#[must_use]
pub fn store(files: &mut Files, expected: file::Expect) -> file::Event {
    files.answer(file::Request::Store {
        owner: super::WORLD_OWNER,
        root: files.roots.cache.token,
        path: FILE_NAME.into(),
        bytes: b"abc".as_slice().into(),
        expected,
        no_follow: true,
    })
}
#[must_use]
pub fn load(files: &mut Files, max: u32) -> file::Event {
    files.answer(file::Request::Load {
        owner: super::WORLD_OWNER,
        root: files.roots.cache.token,
        path: FILE_NAME.into(),
        max,
        no_follow: true,
    })
}

pub const ABC_STORED: file::Event = file::Event::Stored { owner: super::WORLD_OWNER, digest: ABC_SHA256 };
