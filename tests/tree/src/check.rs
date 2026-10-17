//! The typed record oracle and canonical value checks.
use hearth_lib::encode::Encoder;
use hearth_protocol_tree::{
    cache_file::{self, CacheFile},
    decode::Decoder,
    frame::{self, Kind, Unreadable},
    limits::Limits,
    resolution::{self, Resolution},
    store::{self, Store},
};
use skein_lib::Rng;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Record {
    Store(Store),
    CacheFile(CacheFile),
    Resolution(Resolution),
}
impl Record {
    #[must_use]
    pub const fn kind(&self) -> Kind {
        match self {
            Self::Store(_) => Kind::Store,
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
        Kind::CacheFile => cache_file::read(bytes, limits).map(Record::CacheFile),
        Kind::Resolution => resolution::read(bytes, limits).map(Record::Resolution),
    }
}
#[must_use]
pub fn write(record: &Record, limits: &Limits) -> Box<[u8]> {
    match record {
        Record::Store(value) => store::write(value, limits),
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
