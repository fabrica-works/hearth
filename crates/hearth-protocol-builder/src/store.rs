//! Store io, with no waiting in the protocol layer (store.md, 8.2).
use crate::{
    Asked, Roots,
    builder::{
        self,
        store::vocabulary::{Event, Request, Why},
    },
    limits::Limits,
    records, translate, tree,
};
use alloc::boxed::Box;
use skein_io::{file, kernel::Error};
use skein_lib::{Decimal, Queue, Token, Writer, bytes};
pub(super) const OWNER: Token = Token::new(20);
#[derive(Debug)]
pub(super) struct Store {
    pending: Option<Request>,
}
impl Store {
    pub fn new() -> Self {
        Self { pending: None }
    }
    pub fn down(&mut self, request: Request, roots: &Roots, limits: &Limits, below: &mut Queue<Asked>) {
        assert!(self.pending.is_none(), "one store request");
        self.pending = Some(request);
        below.push(Asked::File(match request {
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
    pub fn up(&mut self, event: file::Event, limits: &Limits, above: &mut Queue<builder::vocabulary::Event>) {
        let pending = self.pending.take().expect("store request in flight");
        let event = match event {
            file::Event::Locked { .. } => Event::Locked,
            file::Event::Busy { .. } => Event::Busy,
            file::Event::Loaded { bytes, .. } => Event::Loaded {
                record: match tree::store::read(&bytes, &limits.tree) {
                    Ok(record) => Some(translate::store::store_record(record)),
                    Err(_) => None,
                },
            },
            file::Event::TooLarge { .. } => Event::Loaded { record: None },
            file::Event::Stored { .. } => Event::Done,
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
            | file::Event::Closed { .. }
            | file::Event::Renamed { .. }
            | file::Event::Removed { .. }
            | file::Event::Listed { .. } => unreachable!("store request terminal"),
        };
        above.push(builder::vocabulary::Event::Store(event));
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
