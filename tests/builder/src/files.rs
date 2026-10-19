//! File records played as operations of the fake machine (io.md, 5).
use crate::fixtures::{Fault, STORE, Shared};
use skein_io::{
    digest,
    file::{self, Event, Request},
    kernel::{Error, Fd, OpenHow},
};
use skein_lib::{Rng, Token};
use skein_sim::{Ask, Handle, Reply};
use skein_world::Machine;
use std::collections::BTreeMap;

pub use hearth_protocol_builder::{Root, Roots};
#[derive(Debug)]
pub struct Files {
    pub shared: Shared,
    pub roots: Roots,
    handles: BTreeMap<Token, Handle>,
    next: u64,
    rng: Rng,
    fault: Option<Fault>,
}
impl Files {
    #[must_use]
    pub fn new(mut shared: Shared, fault: Option<Fault>, seed: u64) -> Self {
        let mut handles = BTreeMap::new();
        let mut next = 0;
        let mut root = |path: &[u8]| {
            next += 1;
            let token = Token::new(next);
            handles.insert(token, shared.open_root(path).expect("laid private root"));
            Root { token, fd: Fd::new(i32::try_from(next).expect("root descriptor fits")) }
        };
        let roots = Roots {
            store: root(STORE),
            cache: root(b"/srv/store/cache"),
            resolutions: root(b"/srv/store/resolutions"),
            staging: root(b"/srv/store/staging"),
        };
        Self { shared, roots, handles, next, rng: Rng::new(seed), fault }
    }
    /// The fake machine's handle of the store's root, for the fake tree helper's own calls.
    pub(crate) fn store_root(&self) -> Handle {
        self.handle(self.roots.store.token)
    }
    fn handle(&self, token: Token) -> Handle {
        *self.handles.get(&token).expect("a file token issued here")
    }
    fn fd(&self, fd: Fd) -> Handle {
        self.handle(Token::new(u64::try_from(fd.raw()).expect("a root descriptor")))
    }
    pub(crate) fn call(&self, ask: Ask) -> Result<Reply, Error> {
        self.shared.answer(ask)
    }
    pub(crate) fn open(&self, root: Handle, path: &[u8], how: OpenHow) -> Result<Handle, Error> {
        let Reply::Opened(file) = self.call(Ask::Open { root, path: path.into(), how })? else {
            unreachable!("Open opens")
        };
        Ok(file)
    }
    pub(crate) fn close(&self, file: Handle) {
        self.call(Ask::Close { file }).expect("close an issued file");
    }
    fn opened(&mut self, owner: Token, root: Handle, path: &[u8], how: OpenHow) -> Result<Event, Error> {
        let file = self.open(root, path, how)?;
        let Reply::Stat(stat) = self.call(Ask::Stat { file })? else { unreachable!("Stat states") };
        self.next += 1;
        let token = Token::new(self.next);
        self.handles.insert(token, file);
        Ok(Event::Opened { owner, file: token, len: stat.size })
    }
    /// Every admitted file record ends exactly once; unsupported records identify this world.
    pub fn answer(&mut self, request: Request) -> Event {
        let (owner, result) = self.dispatch(request);
        result.unwrap_or_else(|error| Event::Failed { owner, error, committed: false, residue: None })
    }
    fn dispatch(&mut self, request: Request) -> (Token, Result<Event, Error>) {
        let owner = match &request {
            Request::Lock { owner, .. }
            | Request::MakeDirectory { owner, .. }
            | Request::OpenPrivate { owner, .. }
            | Request::Create { owner, .. }
            | Request::CreateNoFollow { owner, .. }
            | Request::OpenRead { owner, .. }
            | Request::OpenReadNoFollow { owner, .. }
            | Request::OpenWrite { owner, .. }
            | Request::OpenWriteNoFollow { owner, .. }
            | Request::OpenDirectory { owner, .. }
            | Request::Load { owner, .. }
            | Request::Scan { owner, .. }
            | Request::Store { owner, .. }
            | Request::Stat { owner, .. }
            | Request::WriteAt { owner, .. }
            | Request::ReadAt { owner, .. }
            | Request::Cut { owner, .. }
            | Request::Sync { owner, .. }
            | Request::Close { owner, .. }
            | Request::SyncDirectory { owner, .. }
            | Request::Rename { owner, .. }
            | Request::Remove { owner, .. }
            | Request::List { owner, .. }
            | Request::ListPage { owner, .. } => *owner,
        };
        (owner, self.perform(request))
    }
    fn perform(&mut self, request: Request) -> Result<Event, Error> {
        assert!(
            !matches!(
                request,
                Request::OpenPrivate { .. }
                    | Request::Scan { .. }
                    | Request::CreateNoFollow { .. }
                    | Request::OpenRead { .. }
                    | Request::OpenWrite { .. }
                    | Request::OpenWriteNoFollow { .. }
                    | Request::Stat { .. }
                    | Request::Cut { .. }
                    | Request::List { .. }
            ),
            "hearth-28: this layer never asks this file record"
        );
        Ok(match request {
            Request::Lock { owner, root } => match self.call(Ask::Lock { file: self.handle(root) }) {
                Ok(Reply::Done) => Event::Locked { owner },
                Err(Error::Busy) => Event::Busy { owner },
                Err(error) => return Err(error),
                Ok(_) => unreachable!("Lock ends"),
            },
            Request::Load { owner, root, path, max, no_follow } => {
                self.load(owner, self.handle(root), &path, max, no_follow)?
            }
            Request::Store { owner, root, path, bytes, expected, no_follow } => {
                self.store(owner, self.handle(root), &path, &bytes, expected, no_follow)?
            }
            Request::Create { owner, root, name, mode } => {
                self.opened(owner, self.fd(root), &name, OpenHow::Create { mode: Some(mode) })?
            }
            Request::OpenReadNoFollow { owner, root, name } => {
                self.opened(owner, self.fd(root), &name, OpenHow::ReadNoFollow)?
            }
            Request::OpenDirectory { owner, root, name, no_follow } => self.opened(
                owner,
                self.fd(root),
                &name,
                if no_follow { OpenHow::DirectoryNoFollow } else { OpenHow::Directory },
            )?,
            Request::ReadAt { owner, file, offset, max } => {
                let Reply::Read(bytes) = self.call(Ask::Read { file: self.handle(file), at: offset, len: max })? else {
                    unreachable!("Read reads")
                };
                Event::Read { owner, bytes }
            }
            Request::WriteAt { owner, file, offset, bytes } => {
                self.call(Ask::Write { file: self.handle(file), at: offset, bytes })?;
                Event::Written { owner }
            }
            Request::Sync { owner, file } => {
                self.call(Ask::Sync { file: self.handle(file) })?;
                Event::Synced { owner }
            }
            Request::Close { owner, file } => {
                let handle = self.handles.remove(&file).expect("an issued file");
                self.call(Ask::Close { file: handle })?;
                Event::Closed { owner }
            }
            Request::SyncDirectory { owner, root } => {
                self.call(Ask::Sync { file: self.fd(root) })?;
                Event::Synced { owner }
            }
            Request::Rename { owner, from_root, from, to_root, to } => {
                self.call(Ask::Rename {
                    from_dir: self.fd(from_root),
                    from,
                    to_dir: self.fd(to_root),
                    to: to.clone(),
                })?;
                if let Some(Fault::CacheFileGone { sha256 }) = self.fault
                    && to.as_ref() == super::fixtures::hex_digest(sha256.0).as_bytes()
                {
                    self.call(Ask::Remove { dir: self.fd(to_root), name: to, directory: false })?;
                }
                Event::Renamed { owner }
            }
            Request::Remove { owner, root, name, directory } => {
                self.call(Ask::Remove { dir: self.fd(root), name, directory })?;
                Event::Removed { owner }
            }
            Request::ListPage { owner, directory, max } => {
                Event::Listed { owner, entries: self.list(self.handle(directory), max)? }
            }
            Request::MakeDirectory { owner, root, path, mode } => {
                self.call(Ask::MakeDirectory { dir: self.handle(root), name: path, mode })?;
                Event::Made { owner }
            }
            Request::OpenPrivate { .. }
            | Request::Scan { .. }
            | Request::CreateNoFollow { .. }
            | Request::OpenRead { .. }
            | Request::OpenWrite { .. }
            | Request::OpenWriteNoFollow { .. }
            | Request::Stat { .. }
            | Request::Cut { .. }
            | Request::List { .. } => {
                unreachable!("unsupported record was asserted")
            }
        })
    }
    pub(crate) fn list(&self, dir: Handle, max: u32) -> Result<Box<[file::Entry]>, Error> {
        let Reply::Listed(entries) = self.call(Ask::List { dir, most: max, room: max.saturating_mul(256) })? else {
            unreachable!("List lists")
        };
        Ok(entries.into_iter().map(|(kind, name)| file::Entry { kind, name }).collect())
    }
    pub(crate) fn load(
        &self,
        owner: Token,
        root: Handle,
        path: &[u8],
        max: u32,
        no_follow: bool,
    ) -> Result<Event, Error> {
        let file = self.open(root, path, if no_follow { OpenHow::ReadNoFollow } else { OpenHow::Read })?;
        let result = (|| {
            let Reply::Stat(stat) = self.call(Ask::Stat { file })? else { unreachable!("Stat states") };
            if stat.size > u64::from(max) {
                return Ok(Event::TooLarge { owner, size: stat.size });
            }
            let Reply::Read(bytes) = self.call(Ask::Read { file, at: 0, len: max })? else {
                unreachable!("Read reads")
            };
            Ok(Event::Loaded { owner, bytes })
        })();
        self.close(file);
        result
    }
    fn store(
        &mut self,
        owner: Token,
        root: Handle,
        path: &[u8],
        bytes: &[u8],
        expected: file::Expect,
        no_follow: bool,
    ) -> Result<Event, Error> {
        let temporary = [path, format!(".skein-{:016x}", self.rng.next_u64()).as_bytes()].concat();
        let file = self.open(
            root,
            &temporary,
            if no_follow {
                OpenHow::CreateNoFollow { mode: Some(0o600) }
            } else {
                OpenHow::Create { mode: Some(0o600) }
            },
        )?;
        let result =
            self.call(Ask::Write { file, at: 0, bytes: bytes.into() }).and_then(|_| self.call(Ask::Sync { file }));
        self.close(file);
        if let Err(error) = result {
            self.call(Ask::Remove { dir: root, name: temporary.into(), directory: false })?;
            return Err(error);
        }
        if expected != file::Expect::Any {
            let now = match self.load(owner, root, path, u32::MAX, no_follow) {
                Ok(Event::Loaded { bytes, .. }) => Some(digest::digest(&bytes)),
                Err(Error::NotFound) => None,
                Err(error) => return Err(error),
                Ok(_) => unreachable!("unbounded load"),
            };
            let permitted = match expected {
                file::Expect::Any => true,
                file::Expect::Absent => now.is_none(),
                file::Expect::Digest(wanted) => now == Some(wanted),
            };
            if !permitted {
                self.call(Ask::Remove { dir: root, name: temporary.into(), directory: false })?;
                return Ok(Event::Conflict { owner, now });
            }
        }
        self.call(Ask::Rename { from_dir: root, from: temporary.into(), to_dir: root, to: path.into() })?;
        if let Err(error) = self.call(Ask::Sync { file: root }) {
            return Ok(Event::Failed { owner, error, committed: true, residue: None });
        }
        Ok(Event::Stored { owner, digest: digest::digest(bytes) })
    }
    #[must_use]
    pub fn open_files(&self) -> usize {
        self.handles.len()
    }
    #[must_use]
    pub fn names(&self, root: Root) -> Vec<file::Entry> {
        let file = self.open(self.handle(root.token), b".", OpenHow::Directory).expect("directory");
        let entries = self.list(file, 4096).expect("directory entries");
        self.close(file);
        entries.into_vec()
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        for (_, handle) in core::mem::take(&mut self.handles) {
            self.close(handle);
        }
    }
}
