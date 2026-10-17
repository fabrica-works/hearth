//! One value, encoded in pre-order without recursion (store.md, 3.1).

use alloc::boxed::Box;
use skein_lib::{Stack, Writer, bytes};

/// The first byte of `none` (store.md, 3.1).
pub const NONE: u8 = 0x00;
/// The first byte of `false`.
pub const FALSE: u8 = 0x01;
/// The first byte of `true`.
pub const TRUE: u8 = 0x02;
/// The first byte of an unsigned integer.
pub const UINT: u8 = 0x03;
/// The first byte of a byte string, including a map key.
pub const BYTES: u8 = 0x04;
/// The first byte of a digest.
pub const DIGEST: u8 = 0x05;
/// The first byte of a list.
pub const LIST: u8 = 0x06;
/// The first byte of a map.
pub const MAP: u8 = 0x07;
/// The most containers open at once: render.md, 2.3's depth plus a part's levels, with room.
pub const DEPTH: u32 = 64;

/// A SHA-256 digest: a key, a part's digest, or file content (store.md, 3.2).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Digest(pub [u8; 32]);

/// One item in pre-order; a container's contents follow its head.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Item {
    None,
    False,
    True,
    Uint(u64),
    /// Also a key where a map's key belongs.
    Bytes(Box<[u8]>),
    Digest(Digest),
    List(u64),
    Map(u64),
}

/// Encodes one value, asserting its shape as each item arrives (store.md, 3.1).
#[derive(Debug)]
pub struct Encoder {
    sink: Sink,
    open: Stack<Open>,
    placed: bool,
}

#[derive(Debug)]
enum Sink {
    Measure { len: u64 },
    Write { writer: Writer },
}

#[derive(Debug)]
enum Open {
    List { left: u64 },
    Map { left: u64, last: Option<Box<[u8]>>, awaiting: bool },
}

impl Encoder {
    /// Counts the encoded bytes.
    #[must_use]
    pub fn measuring() -> Encoder {
        Encoder { sink: Sink::Measure { len: 0 }, open: Stack::with_capacity(DEPTH), placed: false }
    }

    /// Writes exactly the measured length.
    #[must_use]
    pub fn writing(len: u64) -> Encoder {
        let len = usize::try_from(len).expect("an encoding fits in memory");
        Encoder { sink: Sink::Write { writer: Writer::new(len) }, open: Stack::with_capacity(DEPTH), placed: false }
    }

    pub fn none(&mut self) {
        self.scalar(NONE, &[]);
    }

    pub fn boolean(&mut self, value: bool) {
        self.scalar(if value { TRUE } else { FALSE }, &[]);
    }

    pub fn uint(&mut self, value: u64) {
        self.scalar(UINT, &value.to_be_bytes());
    }

    pub fn bytes(&mut self, value: &[u8]) {
        self.place();
        self.byte_string(value);
        settle(self);
    }

    pub fn digest(&mut self, digest: &Digest) {
        self.scalar(DIGEST, &digest.0);
    }

    /// A list's head; `count` values follow.
    pub fn list(&mut self, count: u64) {
        self.head(LIST, count, Open::List { left: count });
    }

    /// A map's head; `count` entries follow, each a key and a value.
    pub fn map(&mut self, count: u64) {
        self.head(MAP, count, Open::Map { left: count, last: None, awaiting: false });
    }

    /// A map's key, strictly above its previous key bytewise.
    pub fn key(&mut self, key: &[u8]) {
        match self.open.top_mut() {
            Some(Open::Map { last, awaiting, .. }) => {
                assert!(!*awaiting, "a map's key is followed by its value");
                if let Some(previous) = last {
                    assert!(previous.as_ref() < key, "a map's keys strictly increase");
                }
                *last = Some(bytes::copy_of(key));
                *awaiting = true;
            }
            Some(Open::List { .. }) | None => unreachable!("a key belongs to a map"),
        }
        self.byte_string(key);
    }

    /// Writes the item's kind; byte strings in key positions become keys.
    pub fn item(&mut self, item: &Item) {
        match item {
            Item::None => self.none(),
            Item::False => self.boolean(false),
            Item::True => self.boolean(true),
            Item::Uint(value) => self.uint(*value),
            Item::Bytes(value) => match self.open.top() {
                Some(Open::Map { awaiting: false, .. }) => self.key(value),
                Some(Open::Map { awaiting: true, .. } | Open::List { .. }) | None => self.bytes(value),
            },
            Item::Digest(value) => self.digest(value),
            Item::List(count) => self.list(*count),
            Item::Map(count) => self.map(*count),
        }
    }

    #[must_use]
    pub fn measured(self) -> u64 {
        self.whole();
        match self.sink {
            Sink::Measure { len } => len,
            Sink::Write { .. } => unreachable!("a measuring encoder returns its length"),
        }
    }

    #[must_use]
    pub fn written(self) -> Box<[u8]> {
        self.whole();
        match self.sink {
            Sink::Write { writer } => writer.finish(),
            Sink::Measure { .. } => unreachable!("a writing encoder returns its bytes"),
        }
    }

    fn whole(&self) {
        assert!(self.placed && self.open.is_empty(), "an encoding is finished with its one value whole");
    }

    fn place(&mut self) {
        match self.open.top_mut() {
            Some(Open::List { left }) => {
                *left = left.checked_sub(1).expect("a completed list is closed");
            }
            Some(Open::Map { left, awaiting, .. }) => {
                assert!(*awaiting, "a map's entry starts with its key");
                *awaiting = false;
                *left = left.checked_sub(1).expect("a completed map is closed");
            }
            None => {
                assert!(!self.placed, "an encoding holds one value");
                self.placed = true;
            }
        }
    }

    fn scalar(&mut self, kind: u8, value: &[u8]) {
        self.place();
        self.put(&[kind]);
        self.put(value);
        settle(self);
    }

    fn head(&mut self, kind: u8, count: u64, open: Open) {
        self.place();
        self.open.push(open).expect("a value nests at most DEPTH containers deep");
        self.put(&[kind]);
        self.put(&count.to_be_bytes());
        settle(self);
    }

    fn byte_string(&mut self, value: &[u8]) {
        let len = u64::try_from(value.len()).expect("a byte string's length fits in a u64");
        self.put(&[BYTES]);
        self.put(&len.to_be_bytes());
        self.put(value);
    }

    fn put(&mut self, value: &[u8]) {
        match &mut self.sink {
            Sink::Measure { len } => {
                let size = u64::try_from(value.len()).expect("a piece's length fits in a u64");
                *len = len.checked_add(size).expect("an encoding's length fits in a u64");
            }
            Sink::Write { writer } => {
                writer.put(value).expect("an encoder writes within the length it measured");
            }
        }
    }
}

/// A head takes its parent's slot immediately; completed containers close together.
fn settle(encoder: &mut Encoder) {
    for _ in 0..DEPTH {
        match encoder.open.top() {
            Some(Open::List { left: 0 } | Open::Map { left: 0, awaiting: false, .. }) => {
                drop(encoder.open.pop());
            }
            Some(Open::List { .. } | Open::Map { .. }) | None => return,
        }
    }
}
