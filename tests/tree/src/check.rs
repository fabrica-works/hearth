//! The typed record oracle and canonical value checks.
use hearth_lib::encode::Encoder;
use hearth_protocol_tree::{
    cache_file::{self, CacheFile},
    decode::Decoder,
    frame::{self, Kind, Unreadable},
    layer::{self, Layer},
    limits::Limits,
    resolution::{self, Resolution},
    store::{self, Store},
    tree::{self, Item as TreeItem, Next, Reader},
};
use skein_lib::Rng;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Record {
    Store(Store),
    Layer(Box<Layer>),
    Tree(Box<[TreeItem]>),
    CacheFile(CacheFile),
    Resolution(Resolution),
}
impl Record {
    #[must_use]
    pub const fn kind(&self) -> Kind {
        match self {
            Self::Store(_) => Kind::Store,
            Self::Layer(_) => Kind::Layer,
            Self::Tree(_) => Kind::Tree,
            Self::CacheFile(_) => Kind::CacheFile,
            Self::Resolution(_) => Kind::Resolution,
        }
    }
}

/// Reads one record through its kind's real decoder.
///
/// # Errors
/// Returns the decoder's refusal for an incomplete or unreadable frame.
pub fn read(kind: Kind, bytes: &[u8], limits: &Limits) -> Result<Record, Unreadable> {
    match kind {
        Kind::Store => store::read(bytes, limits).map(Record::Store),
        Kind::Layer => layer::read(bytes, limits).map(|record| Record::Layer(Box::new(record))),
        Kind::Tree => read_tree(bytes, limits).map(Record::Tree),
        Kind::CacheFile => cache_file::read(bytes, limits).map(Record::CacheFile),
        Kind::Resolution => resolution::read(bytes, limits).map(Record::Resolution),
    }
}
#[must_use]
pub fn write(record: &Record, limits: &Limits) -> Box<[u8]> {
    match record {
        Record::Store(value) => store::write(value, limits),
        Record::Layer(value) => layer::write(value, limits),
        Record::Tree(items) => Ok(write_tree(items, limits)),
        Record::CacheFile(value) => cache_file::write(value, limits),
        Record::Resolution(value) => resolution::write(value, limits),
    }
    .expect("the generated record is writable")
}

/// Every accepted frame and every accepted body have their one encoding.
pub fn canonical(kind: Kind, bytes: &[u8], limits: &Limits, seed: u64) {
    if let Ok(record) = read(kind, bytes, limits) {
        assert_eq!(write(&record, limits).as_ref(), bytes, "frame, seed {seed}");
    }
    if kind == Kind::Tree {
        return;
    }
    if let Ok(body) = frame::body(bytes, kind, limits) {
        canonical_body(body, limits, seed);
    }
}
/// Every accepted canonical body re-encodes byte for byte.
pub fn canonical_body(body: &[u8], limits: &Limits, seed: u64) {
    let mut decoder = Decoder::new(body, limits);
    let mut items = Vec::new();
    for _ in 0..=body.len() {
        match decoder.item(u32::MAX) {
            Ok(Some(item)) => items.push(item),
            Err(_) => return,
            Ok(None) => {
                let mut writer = Encoder::writing(u64::try_from(body.len()).expect("bounded body"));
                for item in &items {
                    writer.item(item);
                }
                assert_eq!(writer.written().as_ref(), body, "body, seed {seed}");
                return;
            }
        }
    }
    unreachable!("each item consumes at least a byte")
}

pub fn sweep(rng: &mut Rng, record: &Record, limits: &Limits, seed: u64) {
    let frame = write(record, limits);
    assert_eq!(read(record.kind(), &frame, limits), Ok(record.clone()), "seed {seed}");
    // Header checks make every strict prefix fail; sample their bodies in the
    // fuzzy sweep, while the focused test checks every prefix of small frames.
    for _ in 0..8 {
        let cut = usize::try_from(rng.below(u64::try_from(frame.len()).expect("frame size"))).expect("cut");
        assert!(read(record.kind(), &frame[..cut], limits).is_err(), "seed {seed}, cut {cut}");
    }
    for mutation in crate::mutate::frames(rng, &frame) {
        canonical(record.kind(), &mutation, limits, seed);
    }
}

/// Reads a whole tree file the way a walk would, feeding the reader the
/// bytes it asked for and no more, and refusing bytes past the header's length.
///
/// # Errors
/// Returns the reader's refusal, or `Length` for a file that is not as long as
/// its header says.
pub fn read_tree(bytes: &[u8], limits: &Limits) -> Result<Box<[TreeItem]>, Unreadable> {
    let head = <&[u8; 24]>::try_from(bytes.get(..24).ok_or(Unreadable::Short)?).expect("24 bytes");
    let mut reader = Reader::new(limits);
    let length = reader.header(head)?;
    let after = u64::try_from(bytes.len() - 24).expect("a file's size");
    let mut items = Vec::new();
    let (mut at, mut window) = (24_usize, 5_usize);
    for _ in 0..=bytes.len() {
        let fed = &bytes[at..bytes.len().min(at + window)];
        match reader.next(fed)? {
            Next::Item(item, used) => {
                let used = usize::try_from(used).expect("small");
                assert!(used <= fed.len(), "an item takes only what it was fed");
                at += used;
                window = 5;
                items.push(item);
                if reader.ended() {
                    return if at == bytes.len() {
                        Ok(items.into())
                    } else {
                        Err(Unreadable::Length { length, after })
                    };
                }
            }
            Next::Need(need) => {
                let need = usize::try_from(need).expect("small");
                assert!(need > fed.len(), "a reader asks for more than it was given");
                if at + need > bytes.len() {
                    return Err(Unreadable::Length { length, after });
                }
                window = need;
            }
        }
    }
    unreachable!("each step takes a byte or asks for more")
}
/// A tree file: the header with the items' length, then the items.
#[must_use]
pub fn write_tree(items: &[TreeItem], limits: &Limits) -> Box<[u8]> {
    let parts: Vec<Box<[u8]>> =
        items.iter().map(|item| tree::item(item, limits).expect("the generated item is writable")).collect();
    let length: usize = parts.iter().map(|part| part.len()).sum();
    let mut file = tree::header(u64::try_from(length).expect("a file's size")).to_vec();
    for part in &parts {
        file.extend_from_slice(part);
    }
    file.into()
}
