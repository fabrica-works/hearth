//! Store io, with no waiting in the protocol layer (domain/store.md, 8.2).
pub(super) mod names;
use crate::{
    Asked, Roots,
    builder::{
        self,
        store::vocabulary::{Directory, Entry, Event, Helper, Request, Why},
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
    deferred: Option<Event>,
    walk: Option<crate::walk::Walker>,
}
impl Store {
    pub fn new() -> Self {
        Self {
            pending: None,
            cursor: None,
            directory: Directory::Top,
            max: 0,
            closing: None,
            keeping: 0,
            deferred: None,
            walk: None,
        }
    }
    pub fn down(
        &mut self,
        request: Request,
        roots: &Roots,
        limits: &Limits,
        temporary: Option<Box<[u8]>>,
        index: Option<(Token, Box<[u8]>)>,
        below: &mut Queue<Asked>,
    ) {
        assert!(self.pending.is_none(), "one store request");
        self.pending = Some(request.clone());
        if self.write(&request, roots, limits, below)
            || self.index(&request, roots, index, below)
            || self.unavailable(&request)
        {
            return;
        }
        if request == Request::Empty {
            self.walk = Some(crate::walk::Walker::new(limits.walk, roots.staging.fd, below));
            return;
        }
        let removal =
            if let Request::Remove { directory, .. } = &request { removal_root(roots, *directory) } else { None };
        below.push(Asked::File(match request {
            Request::Write { .. } => unreachable!("write handled"),
            Request::Empty => unreachable!("walk started"),
            Request::Create { .. } | Request::Rename { .. } | Request::WriteLayer { .. } | Request::Helper(_) => {
                unreachable!("answered unavailable")
            }
            Request::Load { key } => file::Request::Load {
                owner: OWNER,
                root: roots.resolutions.token,
                path: names::removal(builder::store::vocabulary::Removal::Record(key)),
                max: u32::try_from(tree::frame::largest(tree::frame::Kind::Resolution, &limits.tree))
                    .expect("resolution fits u32"),
                no_follow: true,
            },
            Request::LoadLayer { key } => file::Request::Load {
                owner: OWNER,
                root: roots.store.token,
                path: names::layer_record(key),
                max: u32::try_from(tree::frame::largest(tree::frame::Kind::Layer, &limits.tree))
                    .expect("layer record fits u32"),
                no_follow: true,
            },
            Request::Keep { arrival: _, record } => {
                self.keeping = 1;
                file::Request::Rename {
                    owner: OWNER,
                    from_root: roots.cache.fd,
                    to_root: roots.cache.fd,
                    from: temporary.expect("arrival temporary"),
                    to: names::removal(builder::store::vocabulary::Removal::File(record.sha256)),
                }
            }
            Request::Discard { arrival: _ } => file::Request::Remove {
                owner: OWNER,
                root: roots.cache.fd,
                name: temporary.expect("arrival temporary"),
                directory: false,
            },
            Request::List { directory, max } => {
                self.directory = directory;
                self.max = max;
                file::Request::OpenDirectory {
                    owner: OWNER,
                    root: roots.store.fd,
                    name: bytes::copy_of(match directory {
                        Directory::Top => b".",
                        Directory::Cache => b"cache",
                        Directory::Resolutions => b"resolutions",
                        Directory::Layers => b"layers",
                    }),
                    no_follow: true,
                }
            }
            Request::Next { max } => {
                self.max = max;
                self.list()
            }
            Request::Close => file::Request::Close { owner: OWNER, file: self.cursor.take().expect("scan cursor") },
            Request::Remove { directory: _, name } => file::Request::Remove {
                owner: OWNER,
                root: removal.expect("a removal's root, checked above"),
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
    fn write(&mut self, request: &Request, roots: &Roots, limits: &Limits, below: &mut Queue<Asked>) -> bool {
        if let Request::Write { resolution } = &request {
            match tree::resolution::write(&translate::store::tree_resolution(resolution.clone()), &limits.tree) {
                Ok(bytes) => below.push(Asked::File(file::Request::Store {
                    owner: OWNER,
                    root: roots.resolutions.token,
                    path: names::removal(builder::store::vocabulary::Removal::Record(resolution.key)),
                    bytes,
                    expected: file::Expect::Any,
                    no_follow: true,
                })),
                Err(_) => {
                    self.pending = None;
                    self.deferred =
                        Some(Event::Unwritable { max: u64::from(tree::frame::Kind::Resolution.bound(&limits.tree)) });
                }
            }
            return true;
        }
        false
    }
    fn index(
        &mut self,
        request: &Request,
        roots: &Roots,
        index: Option<(Token, Box<[u8]>)>,
        below: &mut Queue<Asked>,
    ) -> bool {
        if let Some((from_root, from)) = index {
            match &request {
                Request::Keep { record, .. } => {
                    self.keeping = 1;
                    below.push(Asked::Beneath(crate::Beneath::Rename {
                        owner: OWNER,
                        from_root,
                        from,
                        to_root: roots.cache.fd,
                        to: names::removal(builder::store::vocabulary::Removal::File(record.sha256)),
                    }));
                    return true;
                }
                Request::Discard { .. } => {
                    self.pending = None;
                    self.deferred = Some(Event::Done);
                    return true;
                }
                Request::Write { .. }
                | Request::Empty
                | Request::Load { .. }
                | Request::LoadLayer { .. }
                | Request::List { .. }
                | Request::Next { .. }
                | Request::Close
                | Request::Remove { .. }
                | Request::Lock
                | Request::LoadRecord
                | Request::WriteRecord { .. }
                | Request::Create { .. }
                | Request::Rename { .. }
                | Request::WriteLayer { .. }
                | Request::Helper(_) => {}
            }
        }
        false
    }
    /// Answers a request the binary can do nothing for yet `Failed`.
    fn unavailable(&mut self, request: &Request) -> bool {
        if !unavailable(request) {
            return false;
        }
        self.pending = None;
        self.deferred = Some(Event::Failed);
        true
    }
    pub fn ready(&self) -> bool {
        self.deferred.is_some()
    }
    pub fn resume(&mut self, above: &mut Queue<builder::vocabulary::Event>) -> bool {
        if let Some(event) = self.deferred.take() {
            above.push(builder::vocabulary::Event::Store(event));
            true
        } else {
            false
        }
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
        if let Some(walk) = &mut self.walk {
            if let Some(event) = walk.up(event, limits.walk, below) {
                self.walk = None;
                self.pending = None;
                above.push(builder::vocabulary::Event::Store(event));
            }
            return;
        }
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
        let Some(event) = self.terminal(event, limits, &pending, below) else {
            return;
        };
        let event = builder::vocabulary::Event::Store(event);
        if let Request::Keep { .. } | Request::Discard { .. } = pending {
            if let Some(read) = read {
                if let Some(event) = read.settle(event) {
                    above.push(event);
                }
            } else {
                above.push(event);
            }
        } else {
            above.push(event);
        }
    }
    fn terminal(
        &mut self,
        event: file::Event,
        limits: &Limits,
        pending: &Request,
        below: &mut Queue<Asked>,
    ) -> Option<Event> {
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
                    self.pending = Some(pending.clone());
                    below.push(Asked::File(file::Request::Close {
                        owner: OWNER,
                        file: self.cursor.take().expect("scan cursor"),
                    }));
                    return None;
                }
                event
            }
            file::Event::Removed { .. } | file::Event::Closed { .. } | file::Event::Stored { .. } => Event::Done,
            file::Event::Locked { .. } => Event::Locked,
            file::Event::Busy { .. } => Event::Busy,
            file::Event::Loaded { bytes, .. } => loaded(&bytes, pending, limits),
            // A store made by a builder that built no layer has no `layers/` (domain/store.md, 2.4).
            file::Event::Failed { error: Error::NotFound, .. }
                if *pending == Request::List { directory: Directory::Layers, max: self.max } =>
            {
                Event::Page { entries: Box::new([]), last: true }
            }
            file::Event::TooLarge { .. } => {
                if let Request::Load { .. } | Request::LoadLayer { .. } = pending {
                    Event::Unreadable
                } else {
                    Event::Loaded { record: None }
                }
            }
            file::Event::Failed { error: Error::TimedOut, .. } if *pending == Request::Lock => Event::Busy,
            file::Event::Failed { error: Error::NotFound, .. } if *pending == Request::LoadRecord => {
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
        Some(event)
    }
}
/// Whether the binary can do nothing for this request yet; it is answered `Failed`.
fn unavailable(request: &Request) -> bool {
    match request {
        // The protocol holds no root for layers/: nothing is made, renamed, written or removed there
        // before the tasks that do it with btrfs, h08 (domain/store.md, 4.2 and 7.3).
        Request::Create { .. }
        | Request::Rename { .. }
        | Request::WriteLayer { .. }
        | Request::Remove { directory: Directory::Layers, .. } => true,
        // The tree helper is a process of its own, which h06 starts and talks to; until then,
        // each of its requests is unavailable.
        Request::Helper(helper) => match helper {
            Helper::Snapshot { .. }
            | Helper::Walk { .. }
            | Helper::Hash { .. }
            | Helper::Freeze { .. }
            | Helper::Thaw { .. }
            | Helper::Apply { .. }
            | Helper::Empty { .. }
            | Helper::Delete { .. } => true,
        },
        Request::Write { .. }
        | Request::Load { .. }
        | Request::LoadLayer { .. }
        | Request::Empty
        | Request::Keep { .. }
        | Request::Discard { .. }
        | Request::Lock
        | Request::LoadRecord
        | Request::WriteRecord { .. }
        | Request::List { .. }
        | Request::Next { .. }
        | Request::Close
        | Request::Remove { directory: Directory::Top | Directory::Cache | Directory::Resolutions, .. } => false,
    }
}
pub(super) fn startup(why: Why, store: &[u8]) -> records::startup::Startup {
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
    records::startup::Startup::Unopened {
        what: records::startup::Opened::Store,
        path: Some(bytes::copy_of(store)),
        error,
    }
}

fn removal_root(roots: &Roots, directory: Directory) -> Option<skein_io::kernel::Fd> {
    match directory {
        Directory::Top => Some(roots.store.fd),
        Directory::Cache => Some(roots.cache.fd),
        Directory::Resolutions => Some(roots.resolutions.fd),
        Directory::Layers => None,
    }
}
fn loaded(bytes: &[u8], pending: &Request, limits: &Limits) -> Event {
    if let Request::LoadLayer { .. } = pending {
        return match tree::layer::read(bytes, &limits.tree) {
            Ok(record) => Event::Layer { record: translate::store::layer(record) },
            Err(_) => Event::Unreadable,
        };
    }
    if let Request::Load { .. } = pending {
        return match tree::resolution::read(bytes, &limits.tree) {
            Ok(record) => Event::Resolution { record: translate::store::resolution(record) },
            Err(_) => Event::Unreadable,
        };
    }
    Event::Loaded {
        record: match tree::store::read(bytes, &limits.tree) {
            Ok(record) => Some(translate::store::store_record(record)),
            Err(_) => None,
        },
    }
}

impl crate::Protocol {
    pub(super) fn store_request(&mut self, request: Request, limits: &Limits, below: &mut Queue<Asked>) {
        let temporary = self.temporary();
        let index = match &self.resolving {
            Some(resolve) => resolve.index(),
            None => None,
        };
        self.store.down(request, &self.roots, limits, temporary, index, below);
    }

    pub(super) fn resume_store(&mut self, above: &mut Queue<builder::vocabulary::Event>) -> bool {
        if !self.store.resume(above) {
            return false;
        }
        if let Some(resolve) = &mut self.resolving {
            resolve.index_settled();
        }
        true
    }
}
