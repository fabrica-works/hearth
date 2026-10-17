//! Store io, with no waiting in the protocol layer (store.md, 8.2).
pub(super) mod names;
use crate::{
    Asked, Roots,
    builder::{
        self,
        store::vocabulary::{Directory, Entry, Event, Request, Why},
    },
    limits::Limits,
    records, translate, tree,
};
use alloc::boxed::Box;
use skein_io::{file, kernel::Error};
use skein_lib::{Decimal, List, Queue, Token, Writer, bytes};
pub(super) const OWNER: Token = Token::new(20);
#[derive(Debug)]
pub(super) struct Store {
    pending: Option<Request>,
    cursor: Option<Token>,
    directory: Directory,
    max: u32,
    closing: Option<Event>,
    keeping: u8,
}
impl Store {
    pub fn new() -> Self {
        Self { pending: None, cursor: None, directory: Directory::Top, max: 0, closing: None, keeping: 0 }
    }
    pub fn down(
        &mut self,
        request: Request,
        roots: &Roots,
        limits: &Limits,
        read: Option<&mut crate::read::Read>,
        below: &mut Queue<Asked>,
    ) {
        assert!(self.pending.is_none(), "one store request");
        self.pending = Some(request.clone());
        below.push(Asked::File(match request {
            Request::Keep { arrival: _, record } => {
                self.keeping = 1;
                file::Request::Rename {
                    owner: OWNER,
                    from_root: roots.cache.fd,
                    to_root: roots.cache.fd,
                    from: read.expect("arrival read").temporary(),
                    to: names::removal(builder::store::vocabulary::Removal::File(record.sha256)),
                }
            }
            Request::Discard { arrival: _ } => file::Request::Remove {
                owner: OWNER,
                root: roots.cache.fd,
                name: read.expect("arrival read").temporary(),
                directory: false,
            },
            Request::List { directory, max } => {
                self.directory = directory;
                self.max = max;
                file::Request::OpenDirectory {
                    owner: OWNER,
                    root: roots.store.fd,
                    name: bytes::copy_of(if directory == Directory::Top { b"." } else { b"cache" }),
                    no_follow: true,
                }
            }
            Request::Next { max } => {
                self.max = max;
                self.list()
            }
            Request::Close => file::Request::Close { owner: OWNER, file: self.cursor.take().expect("scan cursor") },
            Request::Remove { directory, name } => file::Request::Remove {
                owner: OWNER,
                root: if directory == Directory::Top { roots.store.fd } else { roots.cache.fd },
                name: names::removal(name),
                directory: false,
            },
            Request::Lock => file::Request::Lock { owner: OWNER, root: roots.store.token },
            Request::LoadRecord => file::Request::Load {
                owner: OWNER,
                root: roots.store.token,
                path: bytes::copy_of(b"store.record"),
                max: u32::try_from(tree::frame::largest(tree::frame::Kind::Store, &limits.tree))
                    .expect("store record fits u32"),
                no_follow: true,
            },
            Request::WriteRecord { record } => file::Request::Store {
                owner: OWNER,
                root: roots.store.token,
                path: bytes::copy_of(b"store.record"),
                bytes: tree::store::write(&translate::store::tree_store(record), &limits.tree)
                    .expect("store record within configured bound"),
                expected: file::Expect::Any,
                no_follow: true,
            },
        }));
    }
    fn list(&self) -> file::Request {
        file::Request::ListPage { owner: OWNER, directory: self.cursor.expect("scan cursor"), max: self.max }
    }
    pub fn up(
        &mut self,
        event: file::Event,
        limits: &Limits,
        roots: &Roots,
        read: Option<&mut crate::read::Read>,
        above: &mut Queue<builder::vocabulary::Event>,
        below: &mut Queue<Asked>,
    ) {
        if let Some(Request::Keep { record, .. }) = &self.pending {
            let request = match (&event, self.keeping) {
                (file::Event::Renamed { .. }, 1) => {
                    Some(file::Request::SyncDirectory { owner: OWNER, root: roots.cache.fd })
                }
                (file::Event::Synced { .. }, 2) => Some(file::Request::Store {
                    owner: OWNER,
                    root: roots.cache.token,
                    path: names::removal(builder::store::vocabulary::Removal::Record(record.sha256)),
                    bytes: tree::cache_file::write(&translate::store::cache_file(record.clone()), &limits.tree)
                        .expect("cache record within bound"),
                    expected: file::Expect::Any,
                    no_follow: true,
                }),
                _ => None,
            };
            if let Some(request) = request {
                self.keeping = self.keeping.saturating_add(1);
                below.push(Asked::File(request));
                return;
            }
        }
        if let file::Event::Opened { file, .. } = event {
            self.cursor = Some(file);
            below.push(Asked::File(self.list()));
            return;
        }
        if let file::Event::Closed { .. } = event
            && let Some(event) = self.closing.take()
        {
            self.pending = None;
            above.push(builder::vocabulary::Event::Store(event));
            return;
        }
        if let file::Event::Failed { .. } | file::Event::Cancelled { .. } = event {
            self.closing = None;
            if let Some(file) = self.cursor.take() {
                self.closing = Some(Event::Failed);
                below.push(Asked::File(file::Request::Close { owner: OWNER, file }));
                return;
            }
        }
        let pending = self.pending.take().expect("store request in flight");
        let event = match event {
            file::Event::Listed { entries, .. } => {
                let last = entries.len() < usize::try_from(self.max).expect("page bound");
                let mut shaped = List::with_capacity(self.max);
                for entry in entries {
                    shaped
                        .push(Entry { shape: names::shape(self.directory, &entry.name, entry.kind), name: entry.name })
                        .expect("page bound");
                }
                let event = Event::Page { entries: shaped.into_boxed(), last };
                if last {
                    self.closing = Some(event);
                    self.pending = Some(pending);
                    below.push(Asked::File(file::Request::Close {
                        owner: OWNER,
                        file: self.cursor.take().expect("scan cursor"),
                    }));
                    return;
                }
                event
            }
            file::Event::Removed { .. } | file::Event::Closed { .. } | file::Event::Stored { .. } => Event::Done,
            file::Event::Locked { .. } => Event::Locked,
            file::Event::Busy { .. } => Event::Busy,
            file::Event::Loaded { bytes, .. } => Event::Loaded {
                record: match tree::store::read(&bytes, &limits.tree) {
                    Ok(record) => Some(translate::store::store_record(record)),
                    Err(_) => None,
                },
            },
            file::Event::TooLarge { .. } => Event::Loaded { record: None },
            file::Event::Failed { error: Error::TimedOut, .. } if pending == Request::Lock => Event::Busy,
            file::Event::Failed { error: Error::NotFound, .. } if pending == Request::LoadRecord => {
                Event::Loaded { record: None }
            }
            file::Event::Failed { .. } | file::Event::Cancelled { .. } => Event::Failed,
            file::Event::Opened { .. }
            | file::Event::Made { .. }
            | file::Event::Refused { .. }
            | file::Event::Conflict { .. }
            | file::Event::Scanned { .. }
            | file::Event::Stated { .. }
            | file::Event::Read { .. }
            | file::Event::Written { .. }
            | file::Event::Cut { .. }
            | file::Event::Synced { .. }
            | file::Event::Renamed { .. } => unreachable!("store request terminal"),
        };
        let event = builder::vocabulary::Event::Store(event);
        if let Request::Keep { .. } | Request::Discard { .. } = pending {
            if let Some(event) = read.expect("arrival read").settle(event) {
                above.push(event);
            }
        } else {
            above.push(event);
        }
    }
}
pub(super) fn startup(why: Why) -> records::startup::Startup {
    let error: Box<[u8]> = match why {
        Why::Lock => bytes::copy_of(b"lock failed"),
        Why::Record => bytes::copy_of(b"store.record failed"),
        Why::Owner { recorded, builder } => {
            let recorded = Decimal::of(u64::from(recorded));
            let builder = Decimal::of(u64::from(builder));
            let parts =
                [b"store.record is uid ".as_slice(), recorded.as_bytes(), b"'s, not uid ", builder.as_bytes(), b"'s"];
            let mut length = 0_usize;
            for part in parts {
                length = length.checked_add(part.len()).expect("two u32 uids fit");
            }
            let mut error = Writer::new(length);
            for part in parts {
                error.put(part).expect("two u32 uids fit");
            }
            error.finish()
        }
    };
    records::startup::Startup::Unopened { what: records::startup::Opened::Store, path: None, error }
}
