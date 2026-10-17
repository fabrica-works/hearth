//! Software downloads and scripts; fetch hints stay outside the digest (render.md, 4.2).
pub use super::config::Script;
use crate::{entries::Expansion, errors::Error, revision::declaration::Map};
use alloc::boxed::Box;
use hearth_lib::{encode::Digest, hash::H};
use skein_lib::{List, Map as Ordered};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Software {
    pub downloads: Box<[Download]>,
    pub scripts: Box<[Script]>,
    pub fetch: Box<[Fetch]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Download {
    pub path: Box<[u8]>,
    pub sha256: Digest,
    pub owner: Box<[u8]>,
    pub group: Box<[u8]>,
    pub how: How,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum How {
    File { mode: u16 },
    Tar { strip: u32 },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Fetch {
    pub sha256: Digest,
    pub url: Box<[u8]>,
}
pub fn build(expansion: &Expansion, scripts: Box<[Script]>) -> Result<Software, Box<[Error]>> {
    let entries = expansion.of(Map::Software);
    let count = u32::try_from(entries.len()).expect("entry bound");
    let mut paths = Ordered::with_capacity(count);
    let mut names = Ordered::with_capacity(count);
    let mut faults = List::with_capacity(count);
    for located in &entries {
        if names.contains_key(&located.entry.key) {
            continue;
        } // transitional: hearth 22
        names.insert(located.entry.key.clone(), ()).expect("entry bound");
        match crate::kinds::software::entry(located) {
            Ok((download, fetch)) => {
                if !paths.contains_key(&download.path) {
                    paths.insert(download.path.clone(), (download, fetch)).expect("entry bound");
                }
            } // transitional: hearth 22
            Err(error) => faults.push(*error).expect("entry bound"),
        }
    }
    if !faults.is_empty() {
        return Err(faults.into_boxed());
    }
    let mut downloads = List::with_capacity(count);
    let mut hints = Ordered::with_capacity(count);
    for (_, (download, fetch)) in &paths {
        downloads.push(download.clone()).expect("entry bound");
        if !hints.contains_key(&fetch.sha256) {
            hints.insert(fetch.sha256, fetch.clone()).expect("entry bound");
        }
    }
    let mut fetch = List::with_capacity(hints.len());
    for (_, hint) in &hints {
        fetch.push(hint.clone()).expect("entry bound");
    }
    Ok(Software { downloads: downloads.into_boxed(), scripts, fetch: fetch.into_boxed() })
}
#[must_use]
pub fn digest(software: &Software) -> Digest {
    let mut hash = H::new(b"part:software", 1);
    let record = hash.values();
    record.map(2);
    record.key(b"downloads");
    record.map(u64::try_from(software.downloads.len()).expect("entry bound"));
    for download in &software.downloads {
        record.key(&download.path);
        record.map(5);
        record.key(b"group");
        record.bytes(&download.group);
        if let How::File { mode } = download.how {
            record.key(b"mode");
            record.uint(u64::from(mode));
        }
        record.key(b"owner");
        record.bytes(&download.owner);
        record.key(b"sha256");
        record.digest(&download.sha256);
        if let How::Tar { strip } = download.how {
            record.key(b"strip");
            record.uint(u64::from(strip));
        }
        record.key(b"unpack");
        record.bytes(match download.how {
            How::File { .. } => b"file",
            How::Tar { .. } => b"tar",
        });
    }
    super::config::encode_scripts(&software.scripts, record);
    hash.finish()
}
