//! Empty staging through directory tokens, without following links (store.md, 4.4).
use crate::{Asked, Beneath, builder::store::vocabulary::Event, limits::Walk, store::OWNER};
use alloc::boxed::Box;
use skein_io::{
    file,
    kernel::{Fd, Kind},
};
use skein_lib::{Queue, Stack, Token, bytes};
#[derive(Debug)]
struct Frame {
    token: Token,
    name: Box<[u8]>,
    entries: Box<[file::Entry]>,
    at: usize,
    last: bool,
}
#[derive(Debug)]
pub(super) struct Walker {
    frames: Stack<Frame>,
    opening: Option<Box<[u8]>>,
    closing: Option<Box<[u8]>>,
    count: u32,
    failed: bool,
}
impl Walker {
    pub fn new(limits: Walk, root: Fd, below: &mut Queue<Asked>) -> Self {
        below.push(Asked::File(file::Request::OpenDirectory {
            owner: OWNER,
            root,
            name: bytes::copy_of(b"."),
            no_follow: true,
        }));
        Self {
            frames: Stack::with_capacity(limits.depth),
            opening: Some(bytes::copy_of(b".")),
            closing: None,
            count: 0,
            failed: limits.depth == 0 || limits.page == 0,
        }
    }
    pub fn up(&mut self, event: file::Event, limits: Walk, below: &mut Queue<Asked>) -> Option<Event> {
        match event {
            file::Event::Opened { file, .. } => {
                let name = self.opening.take().expect("walk opening");
                if self.failed {
                    self.closing = Some(name);
                    below.push(Asked::File(file::Request::Close { owner: OWNER, file }));
                    return None;
                }
                self.frames
                    .push(Frame { token: file, name, entries: Box::new([]), at: 0, last: false })
                    .expect("depth checked");
                below.push(Asked::File(file::Request::ListPage { owner: OWNER, directory: file, max: limits.page }));
                return None;
            }
            file::Event::Listed { entries, .. } => {
                self.count = self.count.saturating_add(u32::try_from(entries.len()).expect("page bound"));
                self.failed |= self.count > limits.entries;
                let frame = self.frames.top_mut().expect("walk directory");
                frame.last = entries.len() < usize::try_from(limits.page).expect("page bound");
                frame.entries = entries;
                frame.at = 0;
            }
            file::Event::Closed { .. } => {
                let name = self.closing.take().expect("walk closing");
                if !self.failed {
                    if let Some(parent) = self.frames.top() {
                        below.push(Asked::Beneath(Beneath::Remove {
                            owner: OWNER,
                            root: parent.token,
                            name,
                            directory: true,
                        }));
                        return None;
                    }
                    return Some(Event::Done);
                }
            }
            file::Event::Removed { .. } => {}
            file::Event::Failed { .. } | file::Event::Cancelled { .. } => {
                self.failed = true;
            }
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
            | file::Event::Written { .. }
            | file::Event::Read { .. }
            | file::Event::Cut { .. }
            | file::Event::Synced { .. }
            | file::Event::Renamed { .. } => unreachable!("walk terminal"),
        }
        if self.failed {
            return self.close(below);
        }
        let frame = self.frames.top_mut().expect("walk directory");
        if let Some(entry) = frame.entries.get(frame.at) {
            frame.at = frame.at.saturating_add(1);
            if entry.kind == Kind::Directory {
                if self.frames.len() >= limits.depth {
                    self.failed = true;
                    return self.close(below);
                }
                let frame = self.frames.top().expect("walk directory");
                let entry = frame.entries.get(frame.at.saturating_sub(1)).expect("entry cursor");
                self.opening = Some(entry.name.clone());
                below.push(Asked::Beneath(Beneath::OpenDirectory {
                    owner: OWNER,
                    root: frame.token,
                    name: entry.name.clone(),
                    no_follow: true,
                }));
            } else {
                below.push(Asked::Beneath(Beneath::Remove {
                    owner: OWNER,
                    root: frame.token,
                    name: entry.name.clone(),
                    directory: false,
                }));
            }
            return None;
        }
        if !frame.last {
            below.push(Asked::File(file::Request::ListPage { owner: OWNER, directory: frame.token, max: limits.page }));
            return None;
        }
        self.close(below)
    }
    fn close(&mut self, below: &mut Queue<Asked>) -> Option<Event> {
        if let Some(frame) = self.frames.pop() {
            self.closing = Some(frame.name);
            below.push(Asked::File(file::Request::Close { owner: OWNER, file: frame.token }));
            None
        } else {
            Some(Event::Failed)
        }
    }
}
