//! Domain store records retain the file layer's operations and codecs.
use crate::{
    files::{Files, Root},
    fixtures::{WORLD_OWNER, hex_digest},
};
use hearth_domain_store::vocabulary::{Directory, Entry, Event, Removal, Request, Shape};
use hearth_lib::encode::Digest;
use hearth_protocol_builder::translate;
use hearth_protocol_tree::{self as tree, limits::SHIPPED};
use skein_io::{
    file,
    kernel::{Fd, Kind},
};
use skein_lib::Token;
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub(super) struct Store {
    cursor: Option<Token>,
    directory: Option<Directory>,
    arrivals: BTreeMap<Token, Box<[u8]>>,
    next: u64,
}
impl Store {
    pub fn arrival(&mut self, files: &mut Files, sha256: Digest, bytes: &[u8]) -> Token {
        self.next += 1;
        let token = Token::new(self.next);
        let name: Box<[u8]> = format!("{}.tmp", hex_digest(sha256.0)).into_bytes().into();
        let file::Event::Opened { file, .. } = files.answer(file::Request::Create {
            owner: WORLD_OWNER,
            root: files.roots.cache.fd,
            name: name.clone(),
            mode: 0o600,
        }) else {
            unreachable!("arrival temporary opens")
        };
        assert_eq!(
            files.answer(file::Request::WriteAt { owner: WORLD_OWNER, file, offset: 0, bytes: bytes.into() }),
            file::Event::Written { owner: WORLD_OWNER }
        );
        assert_eq!(
            files.answer(file::Request::Sync { owner: WORLD_OWNER, file }),
            file::Event::Synced { owner: WORLD_OWNER }
        );
        assert_eq!(
            files.answer(file::Request::Close { owner: WORLD_OWNER, file }),
            file::Event::Closed { owner: WORLD_OWNER }
        );
        self.arrivals.insert(token, name);
        token
    }
    pub fn answer(&mut self, files: &mut Files, request: Request) -> Event {
        match request {
            Request::Empty => {
                empty(files);
                Event::Done
            }
            Request::List { directory, max } => self.list(files, directory, max),
            Request::Next { max } => self.page(files, max),
            Request::Close => {
                self.close(files);
                Event::Done
            }
            Request::Keep { arrival, record } => self.keep(files, arrival, record),
            Request::Discard { arrival } => {
                let name = self.arrivals.remove(&arrival).expect("an arrival in flight");
                terminal(&files.answer(file::Request::Remove {
                    owner: WORLD_OWNER,
                    root: files.roots.cache.fd,
                    name,
                    directory: false,
                }))
            }
            Request::Lock => {
                let event = files.answer(file::Request::Lock { owner: WORLD_OWNER, root: files.roots.store.token });
                if let file::Event::Locked { .. } = event {
                    Event::Locked
                } else if let file::Event::Busy { .. } = event {
                    Event::Busy
                } else {
                    Event::Failed
                }
            }
            Request::LoadRecord => {
                let event = load(files, files.roots.store, b"store.record", tree::frame::Kind::Store);
                if let file::Event::Loaded { bytes, .. } = event {
                    Event::Loaded {
                        record: tree::store::read(&bytes, &SHIPPED).ok().map(translate::store::store_record),
                    }
                } else if let file::Event::Failed { error: skein_io::kernel::Error::NotFound, .. } = event {
                    Event::Loaded { record: None }
                } else {
                    Event::Failed
                }
            }
            Request::WriteRecord { record } => {
                let bytes = tree::store::write(&translate::store::tree_store(record), &SHIPPED).expect("store record");
                terminal(&write(files, files.roots.store, b"store.record", bytes))
            }
            Request::Remove { directory, name } => {
                let name = match name {
                    Removal::Named(name) => name,
                    Removal::File(digest) => hex_digest(digest.0).into_bytes().into(),
                    Removal::Record(digest) => format!("{}.record", hex_digest(digest.0)).into_bytes().into(),
                };
                let Some(root) = root(files, directory) else {
                    return Event::Failed;
                };
                terminal(&files.answer(file::Request::Remove {
                    owner: WORLD_OWNER,
                    root: root.fd,
                    name,
                    directory: false,
                }))
            }
            Request::Load { key } => {
                let event = load(
                    files,
                    files.roots.resolutions,
                    format!("{}.record", hex_digest(key.0)).as_bytes(),
                    tree::frame::Kind::Resolution,
                );
                if let file::Event::Loaded { bytes, .. } = event {
                    match tree::resolution::read(&bytes, &SHIPPED) {
                        Ok(record) => Event::Resolution { record: translate::store::resolution(record) },
                        Err(_) => Event::Unreadable,
                    }
                } else {
                    Event::Unreadable
                }
            }
            Request::LoadLayer { key } => {
                let path = format!("layers/{}.record", hex_digest(key.0));
                let event = load(files, files.roots.store, path.as_bytes(), tree::frame::Kind::Layer);
                if let file::Event::Loaded { bytes, .. } = event {
                    match tree::layer::read(&bytes, &SHIPPED) {
                        Ok(record) => Event::Layer { record: translate::store::layer(record) },
                        Err(_) => Event::Unreadable,
                    }
                } else {
                    Event::Unreadable
                }
            }
            Request::Write { resolution } => {
                let name = format!("{}.record", hex_digest(resolution.key.0));
                let bytes = tree::resolution::write(&translate::store::tree_resolution(resolution), &SHIPPED)
                    .expect("drawn closure within shipped limits");
                terminal(&write(files, files.roots.resolutions, name.as_bytes(), bytes))
            }
        }
    }
    fn keep(&mut self, files: &mut Files, arrival: Token, record: hearth_domain_store::vocabulary::CacheFile) -> Event {
        let from = self.arrivals.remove(&arrival).expect("an arrival in flight");
        let to = hex_digest(record.sha256.0).into_bytes().into();
        let cache = files.roots.cache;
        if files.answer(file::Request::Rename { owner: WORLD_OWNER, from_root: cache.fd, from, to_root: cache.fd, to })
            != (file::Event::Renamed { owner: WORLD_OWNER })
        {
            return Event::Failed;
        }
        if files.answer(file::Request::SyncDirectory { owner: WORLD_OWNER, root: cache.fd })
            != (file::Event::Synced { owner: WORLD_OWNER })
        {
            return Event::Failed;
        }
        let name = format!("{}.record", hex_digest(record.sha256.0));
        let bytes = tree::cache_file::write(&translate::store::cache_file(record), &SHIPPED)
            .expect("cache record within shipped limits");
        terminal(&write(files, cache, name.as_bytes(), bytes))
    }
    /// `layers/` is opened beneath the store's root, and absent in a store without layers.
    fn list(&mut self, files: &mut Files, directory: Directory, max: u32) -> Event {
        self.directory = Some(directory);
        let (root, name): (Root, &[u8]) = match root(files, directory) {
            Some(root) => (root, b"."),
            None => (files.roots.store, b"layers"),
        };
        let opened = files.answer(file::Request::OpenDirectory {
            owner: WORLD_OWNER,
            root: root.fd,
            name: name.into(),
            no_follow: true,
        });
        if let file::Event::Failed { error: skein_io::kernel::Error::NotFound, .. } = opened
            && directory == Directory::Layers
        {
            return Event::Page { entries: Box::new([]), last: true };
        }
        let file::Event::Opened { file, .. } = opened else {
            return Event::Failed;
        };
        self.cursor = Some(file);
        self.page(files, max)
    }
    fn page(&mut self, files: &mut Files, max: u32) -> Event {
        let file::Event::Listed { entries, .. } = files.answer(file::Request::ListPage {
            owner: WORLD_OWNER,
            directory: self.cursor.expect("scan cursor"),
            max,
        }) else {
            return Event::Failed;
        };
        let last = entries.len() < usize::try_from(max).expect("page bound fits");
        let directory = self.directory.expect("scan directory");
        let entries = entries
            .into_iter()
            .map(|entry| Entry { shape: shape(directory, &entry.name, entry.kind), name: entry.name })
            .collect();
        if last {
            self.close(files);
        }
        Event::Page { entries, last }
    }
    fn close(&mut self, files: &mut Files) {
        if let Some(file) = self.cursor.take() {
            assert_eq!(
                files.answer(file::Request::Close { owner: WORLD_OWNER, file }),
                file::Event::Closed { owner: WORLD_OWNER }
            );
        }
    }
}
/// The roots the world opens; `layers/` has none, as in the protocol layer.
fn root(files: &Files, directory: Directory) -> Option<Root> {
    match directory {
        Directory::Top => Some(files.roots.store),
        Directory::Cache => Some(files.roots.cache),
        Directory::Resolutions => Some(files.roots.resolutions),
        Directory::Layers => None,
    }
}
fn load(files: &mut Files, root: Root, name: &[u8], kind: tree::frame::Kind) -> file::Event {
    files.answer(file::Request::Load {
        owner: WORLD_OWNER,
        root: root.token,
        path: name.into(),
        max: u32::try_from(tree::frame::largest(kind, &SHIPPED)).expect("frame bound fits"),
        no_follow: true,
    })
}
fn write(files: &mut Files, root: Root, path: &[u8], bytes: Box<[u8]>) -> file::Event {
    files.answer(file::Request::Store {
        owner: WORLD_OWNER,
        root: root.token,
        path: path.into(),
        bytes,
        expected: file::Expect::Any,
        no_follow: true,
    })
}
fn terminal(event: &file::Event) -> Event {
    if matches!(event, file::Event::Stored { .. } | file::Event::Removed { .. }) { Event::Done } else { Event::Failed }
}
fn shape(directory: Directory, name: &[u8], kind: Kind) -> Shape {
    if directory == Directory::Layers {
        return hearth_protocol_builder::store_shape(directory, name, kind);
    }
    if directory == Directory::Top
        && matches!(
            name,
            b"store.record"
                | b"cache"
                | b"resolutions"
                | b"staging"
                | b"layers"
                | b"targets"
                | b"state"
                | b"artifacts"
                | b"sources"
                | b"caches"
                | b"repo"
                | b"credentials"
        )
    {
        return Shape::Fixed;
    }
    if kind != Kind::File {
        return Shape::Stray;
    }
    if name.ends_with(b".tmp") || name.windows(7).any(|part| part == b".skein-") {
        return Shape::Temporary;
    }
    let prefix = name.strip_suffix(b".record").unwrap_or(name);
    let Ok(hex) = <&[u8; 64]>::try_from(prefix) else {
        return Shape::Stray;
    };
    let mut bytes = [0; 32];
    for (byte, pair) in bytes.iter_mut().zip(hex.chunks_exact(2)) {
        let [high, low] = pair else { unreachable!("two digits") };
        let (Some(high), Some(low)) = (char::from(*high).to_digit(16), char::from(*low).to_digit(16)) else {
            return Shape::Stray;
        };
        *byte = u8::try_from(high * 16 + low).expect("two hex digits");
    }
    if name.ends_with(b".record") {
        Shape::Record(Digest(bytes))
    } else if directory == Directory::Cache {
        Shape::File(Digest(bytes))
    } else {
        Shape::Stray
    }
}
fn empty(files: &mut Files) {
    let mut pending = vec![(files.roots.staging.fd, None)];
    for _ in 0..4096 {
        let Some((root, remove)) = pending.pop() else {
            return;
        };
        if let Some((parent, name)) = remove {
            assert_eq!(
                files.answer(file::Request::Close {
                    owner: WORLD_OWNER,
                    file: Token::new(u64::try_from(root.raw()).expect("directory token"))
                }),
                file::Event::Closed { owner: WORLD_OWNER }
            );
            assert_eq!(
                files.answer(file::Request::Remove { owner: WORLD_OWNER, root: parent, name, directory: true }),
                file::Event::Removed { owner: WORLD_OWNER }
            );
            continue;
        }
        let file::Event::Opened { file, .. } = files.answer(file::Request::OpenDirectory {
            owner: WORLD_OWNER,
            root,
            name: b".".as_slice().into(),
            no_follow: true,
        }) else {
            unreachable!("staging directory")
        };
        let file::Event::Listed { entries, .. } =
            files.answer(file::Request::ListPage { owner: WORLD_OWNER, directory: file, max: 4096 })
        else {
            unreachable!("staging listing")
        };
        files.answer(file::Request::Close { owner: WORLD_OWNER, file });
        for entry in entries {
            if entry.kind == Kind::Directory {
                let file::Event::Opened { file, .. } = files.answer(file::Request::OpenDirectory {
                    owner: WORLD_OWNER,
                    root,
                    name: entry.name.clone(),
                    no_follow: true,
                }) else {
                    unreachable!("staging child")
                };
                let fd = Fd::new(i32::try_from(file.raw()).expect("world fd"));
                pending.push((fd, Some((root, entry.name))));
                pending.push((fd, None));
            } else {
                assert_eq!(
                    files.answer(file::Request::Remove {
                        owner: WORLD_OWNER,
                        root,
                        name: entry.name,
                        directory: false
                    }),
                    file::Event::Removed { owner: WORLD_OWNER }
                );
            }
        }
    }
    unreachable!("bounded staging walk")
}
