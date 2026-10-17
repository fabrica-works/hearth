//! JSON lines, one measured Encoder document per line (client/README.md, 3.2).
use super::Refusal;
use crate::{
    CLIENT_VERSION,
    outcome::Outcome,
    records::{Record, Type},
};
use alloc::boxed::Box;
use skein_json::writer::{Encoder, Limits, Refusal as JsonRefusal};
use skein_lib::{List, Writer};
pub(crate) const DEPTH: u32 = 64;
pub(crate) fn print(record: &Record, after: Option<Type>, most: u32) -> Result<Box<[u8]>, Refusal> {
    let limits = Limits { depth: DEPTH, length: most };
    let count = match record {
        Record::Spec(spec) => spec.parts.len().checked_add(3),
        Record::Terminal(_)
        | Record::Change(_)
        | Record::Error(_)
        | Record::Startup(_)
        | Record::Waiting(_)
        | Record::Strays(_) => Some(2),
    }
    .ok_or(Refusal::TooLong)?;
    let start = usize::from(after.is_some());
    let mut lengths = List::with_capacity(match u32::try_from(count) {
        Ok(count) => count,
        Err(_) => return Err(Refusal::TooLong),
    });
    let mut total = 0_usize;
    for index in start..count {
        let mut json = Encoder::measure(&limits);
        line(&mut json, record, index);
        let len = match json.measured() {
            Ok(len) => len,
            Err(JsonRefusal::Text) => return Err(Refusal::Text),
            Err(JsonRefusal::TooLong) => return Err(Refusal::TooLong),
            Err(JsonRefusal::Number | JsonRefusal::TooDeep) => {
                unreachable!("typed records use strings and bounded nesting")
            }
        };
        total = total.checked_add(usize::try_from(len).expect("u32 fits usize")).ok_or(Refusal::TooLong)?;
        total = total.checked_add(1).ok_or(Refusal::TooLong)?;
        lengths.push(len).expect("one length per line");
    }
    if total > usize::try_from(most).expect("u32 fits usize") {
        return Err(Refusal::TooLong);
    }
    let mut writer = Writer::new(total);
    for (index, len) in (start..count).zip(lengths.as_slice()) {
        let mut json = Encoder::write(*len, &limits);
        line(&mut json, record, index);
        writer.put(&json.finish()).expect("measured JSON fits");
        writer.put(b"\n").expect("measured LF fits");
    }
    Ok(writer.finish())
}
pub(crate) fn field(json: &mut Encoder, key: &[u8], value: &[u8]) {
    json.key(key);
    json.string(value);
}
fn line(json: &mut Encoder, record: &Record, index: usize) {
    json.object_start();
    if index == 0 {
        field(json, b"record", b"version");
        json.key(b"client");
        json.unsigned(u64::from(CLIENT_VERSION));
    } else {
        match record {
            Record::Waiting(waiting) => waiting.json(json),
            Record::Strays(strays) => strays.json(json),
            Record::Change(change) => change.json(json),
            Record::Error(error) => error.json(json),
            Record::Startup(startup) => startup.json(json),
            Record::Terminal(terminal) => {
                field(json, b"record", b"outcome");
                field(
                    json,
                    b"outcome",
                    match terminal.outcome {
                        Outcome::Done => b"done",
                        Outcome::Negative => b"negative",
                        Outcome::Failed => b"failed",
                        Outcome::Usage => b"usage",
                        Outcome::TryAgain => b"try_again",
                    },
                );
                if let Some(tally) = &terminal.diff {
                    tally.json(json);
                }
                json.key(b"left_out");
                json.unsigned(u64::from(terminal.left_out));
            }
            Record::Spec(spec) => {
                if index == 1 {
                    field(json, b"record", b"spec");
                    field(json, b"target", &spec.meta.target);
                    field(json, b"commit", &spec.commit);
                    json.key(b"digest");
                    digest(json, &spec.digest);
                    if let Some(value) = &spec.resolution {
                        json.key(b"resolution");
                        resolution(json, value);
                    }
                } else {
                    field(json, b"record", b"part");
                    field(json, b"target", &spec.meta.target);
                    if index == 2 {
                        field(json, b"name", b"meta");
                        json.key(b"digest");
                        digest(json, &spec.meta_digest);
                        json.key(b"meta");
                        spec.meta.json(json);
                    } else {
                        let part = spec.parts.get(index.saturating_sub(3)).expect("one line per part");
                        field(json, b"name", part.record.name());
                        json.key(b"digest");
                        digest(json, &part.digest);
                        json.key(part.record.name());
                        part.record.json(json);
                    }
                }
            }
        }
    }
    json.object_end();
}
pub(crate) fn digest(json: &mut Encoder, value: &[u8; 32]) {
    json.string(&digest_hex(value));
}
pub(crate) fn digest_hex(value: &[u8; 32]) -> [u8; 64] {
    let mut digits = [0_u8; 64];
    for (pair, byte) in digits.chunks_exact_mut(2).zip(value) {
        for (digit, nibble) in pair.iter_mut().zip([byte >> 4_u32, byte & 15]) {
            *digit =
                if nibble < 10 { b'0'.saturating_add(nibble) } else { b'a'.saturating_add(nibble.saturating_sub(10)) };
        }
    }
    digits
}

fn resolution(json: &mut Encoder, resolution: &crate::records::Resolution) {
    use crate::records::Resolution;
    json.object_start();
    match resolution {
        Resolution::Needed { key } => {
            json.key(b"key");
            digest(json, key);
            field(json, b"kind", b"needed");
        }
        Resolution::Known { key, keys } => {
            json.key(b"key");
            digest(json, key);
            json.key(b"keys");
            json.object_start();
            json.key(b"config");
            digest(json, &keys.config);
            json.key(b"packages");
            digest(json, &keys.packages);
            json.key(b"software");
            digest(json, &keys.software);
            json.object_end();
            field(json, b"kind", b"known");
        }
    }
    json.object_end();
}
