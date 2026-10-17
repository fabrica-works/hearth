use crate::config::{
    Position,
    document::{Document, Layer},
    json::{Entry, Kind, Value},
};
use skein_lib::bytes::copy_of;
#[must_use]
pub fn debian_documents() -> [Document; 2] {
    [b"forky".as_slice(), b"trixie"].map(|suite| {
        let at = Position { offset: 0, line: 1, column: 1 };
        Document {
            at,
            members: None,
            layer: Layer {
                tasks: Box::new([]),
                placement: None,
                debian: Some(Value {
                    at,
                    kind: Kind::Map(Box::new([Entry {
                        key: copy_of(b"suite"),
                        at,
                        value: Value { at, kind: Kind::String(copy_of(suite)) },
                    }])),
                }),
            },
        }
    })
}
