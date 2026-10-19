//! Forms shared by path and seed changes (domain/render.md, 6.1).
use crate::form::{json::field, text::Lines};
use skein_json::writer::Encoder;
pub(crate) fn header(lines: &mut Lines, part: &[u8], key: &[u8], before: bool, after: bool) {
    lines.put(b"  ");
    lines.put(part);
    lines.pad_to(11);
    lines.put(match (before, after) {
        (false, true) => b"+ ",
        (true, false) => b"- ",
        (true, true) => b"~ ",
        (false, false) => unreachable!("change has a side"),
    });
    lines.put(key);
}
pub(crate) fn keyed<T: ?Sized>(
    json: &mut Encoder,
    key: &[u8],
    before: Option<&T>,
    after: Option<&T>,
    encode: fn(&T, &mut Encoder),
) {
    json.object_start();
    match (before, after) {
        (Some(before), Some(after)) => {
            json.key(b"after");
            encode(after, json);
            json.key(b"before");
            encode(before, json);
            field(json, b"kind", b"changed");
        }
        (None, Some(value)) => {
            field(json, b"kind", b"added");
            json.key(b"value");
            encode(value, json);
        }
        (Some(value), None) => {
            field(json, b"kind", b"removed");
            json.key(b"value");
            encode(value, json);
        }
        (None, None) => unreachable!("change has a side"),
    }
    field(json, b"key", key);
    json.object_end();
}
