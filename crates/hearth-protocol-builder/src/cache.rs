//! A single streamed cache file, durable before publication (store.md, 6.8).
use crate::render::vocabulary::Digest;
use alloc::boxed::Box;
use skein_io::{
    file,
    kernel::{Error, Fd},
};
use skein_lib::{Rng, Token, Writer, sha256::Sha256};
const OWNER: Token = Token::new(10);
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Stage {
    Creating,
    Ready,
    Writing,
    Syncing,
    Closing,
    Checking,
    Existing,
    Removing,
    Renaming,
    Directory,
    Done,
    Failed,
}
#[derive(Debug)]
pub(crate) struct Cache {
    root: Fd,
    name: Box<[u8]>,
    hash: Sha256,
    digest: Option<Digest>,
    file: Option<Token>,
    offset: u64,
    stage: Stage,
    pending: Option<file::Request>,
    written: bool,
    failing: bool,
}
impl Cache {
    pub fn new(root: Fd, rng: &mut Rng) -> Self {
        let mut name = Writer::new(68);
        for _ in 0_u8..4 {
            for byte in rng.next_u64().to_be_bytes() {
                hex(&mut name, byte);
            }
        }
        name.put(b".tmp").expect("temporary suffix");
        let name = name.finish();
        let pending = Some(file::Request::Create { owner: OWNER, root, name: name.clone(), mode: 0o666 });
        Self {
            root,
            name,
            hash: Sha256::new(),
            digest: None,
            file: None,
            offset: 0,
            stage: Stage::Creating,
            pending,
            written: false,
            failing: false,
        }
    }
    pub fn take_request(&mut self) -> Option<file::Request> {
        self.pending.take()
    }
    pub fn work_pending(&self) -> bool {
        self.pending.is_some() || self.written
    }
    pub fn readable(&self) -> bool {
        self.stage == Stage::Ready
    }
    pub fn busy(&self) -> bool {
        !self.readable()
    }
    pub fn failed(&self) -> bool {
        self.stage == Stage::Failed
    }
    pub fn done(&self) -> bool {
        self.stage == Stage::Done
    }
    pub fn digest(&self) -> Digest {
        self.digest.expect("finished hash")
    }
    pub fn written(&mut self) -> bool {
        core::mem::replace(&mut self.written, false)
    }
    pub fn piece(&mut self, bytes: Box<[u8]>) {
        assert!(self.readable(), "write one piece at a time");
        self.hash.update(&bytes);
        self.pending = Some(file::Request::WriteAt {
            owner: OWNER,
            file: self.file.expect("created file"),
            offset: self.offset,
            bytes,
        });
        if let Some(file::Request::WriteAt { bytes, .. }) = &self.pending {
            self.offset =
                self.offset.checked_add(u64::try_from(bytes.len()).expect("piece length")).expect("bounded file");
        }
        self.stage = Stage::Writing;
    }
    pub fn finish(&mut self) {
        self.digest = Some(Digest(core::mem::replace(&mut self.hash, Sha256::new()).finish()));
        self.stage = Stage::Syncing;
        self.pending = Some(file::Request::Sync { owner: OWNER, file: self.file.expect("created file") });
    }
    pub fn closing(&self) -> bool {
        self.failing
    }
    pub fn abort(&mut self) -> bool {
        if self.done() || self.failed() {
            return true;
        }
        if !self.failing {
            self.failing = true;
            if self.pending.is_some() || self.readable() {
                self.pending = None;
                self.close_failed();
            }
        }
        self.failed()
    }
    fn close_failed(&mut self) {
        self.written = false;
        if let Some(file) = self.file {
            self.stage = Stage::Closing;
            self.pending = Some(file::Request::Close { owner: OWNER, file });
        } else {
            self.stage = Stage::Failed;
        }
    }
    pub fn up(&mut self, event: file::Event) {
        assert_eq!(event.owner(), OWNER, "cache owner");
        if self.failing {
            if let file::Event::Opened { file, .. } = &event {
                self.file = Some(*file);
            }
            if let file::Event::Closed { .. } = &event {
                self.file = None;
            }
            if let file::Event::Failed { .. } = &event
                && self.stage == Stage::Closing
            {
                self.file = None;
            }
            self.close_failed();
            return;
        }
        match event {
            file::Event::Opened { file, .. } => {
                self.file = Some(file);
                if self.stage == Stage::Creating {
                    self.stage = Stage::Ready;
                } else {
                    assert_eq!(self.stage, Stage::Checking, "checking existing file");
                    self.stage = Stage::Existing;
                    self.pending = Some(file::Request::Close { owner: OWNER, file });
                }
            }
            file::Event::Written { .. } => {
                self.stage = Stage::Ready;
                self.written = true;
            }
            file::Event::Synced { .. } => {
                if self.stage == Stage::Directory {
                    self.stage = Stage::Done;
                } else {
                    self.stage = Stage::Closing;
                    self.pending = Some(file::Request::Close { owner: OWNER, file: self.file.expect("temporary") });
                }
            }
            file::Event::Closed { .. } => {
                self.file = None;
                if self.failing {
                    self.stage = Stage::Failed;
                } else if self.stage == Stage::Existing {
                    self.stage = Stage::Removing;
                    self.pending = Some(file::Request::Remove {
                        owner: OWNER,
                        root: self.root,
                        name: self.name.clone(),
                        directory: false,
                    });
                } else {
                    self.stage = Stage::Checking;
                    self.pending =
                        Some(file::Request::OpenRead { owner: OWNER, root: self.root, name: self.digest_name() });
                }
            }
            file::Event::Failed { error: Error::NotFound, .. } if self.stage == Stage::Checking => {
                self.stage = Stage::Renaming;
                self.pending = Some(file::Request::Rename {
                    owner: OWNER,
                    from_root: self.root,
                    to_root: self.root,
                    from: self.name.clone(),
                    to: self.digest_name(),
                });
            }
            file::Event::Failed { .. } => {
                self.failing = true;
                if let Some(file) = self.file {
                    self.pending = Some(file::Request::Close { owner: OWNER, file });
                } else {
                    self.stage = Stage::Failed;
                }
            }
            file::Event::Renamed { .. } => {
                self.stage = Stage::Directory;
                self.pending = Some(file::Request::SyncDirectory { owner: OWNER, root: self.root });
            }
            file::Event::Removed { .. } => self.stage = Stage::Done,
            file::Event::Locked { .. }
            | file::Event::Busy { .. }
            | file::Event::Made { .. }
            | file::Event::Refused { .. }
            | file::Event::Loaded { .. }
            | file::Event::Scanned { .. }
            | file::Event::Stored { .. }
            | file::Event::Conflict { .. }
            | file::Event::TooLarge { .. }
            | file::Event::Stated { .. }
            | file::Event::Read { .. }
            | file::Event::Cut { .. }
            | file::Event::Listed { .. }
            | file::Event::Cancelled { .. } => unreachable!("cache operations only"),
        }
    }
    fn digest_name(&self) -> Box<[u8]> {
        let mut name = Writer::new(64);
        for byte in self.digest().0 {
            hex(&mut name, byte);
        }
        name.finish()
    }
}
fn hex(writer: &mut Writer, byte: u8) {
    for digit in [byte >> 4_u8, byte & 15] {
        writer
            .put(&[if digit < 10 { b'0'.saturating_add(digit) } else { b'a'.saturating_add(digit.saturating_sub(10)) }])
            .expect("hex room");
    }
}
