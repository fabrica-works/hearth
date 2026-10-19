use super::{Box, Composed, Documents, Kind, Layer, Merge, Position, Target, Value, bytes, layers, revision};
/// Compose the fixed Debian schema over protocol/format.md, 3.2's six layers.
#[must_use]
pub fn debian(target: &Target, documents: &Documents, merge: &mut Merge) -> Option<Composed> {
    let at = Position { line: 1, column: 1 };
    let bottom = Value {
        at,
        kind: Kind::Map(Box::new([revision::Pair {
            key: bytes::copy_of(b"components"),
            at,
            value: Value {
                at,
                kind: Kind::Map(Box::new([revision::Pair {
                    key: bytes::copy_of(b"main"),
                    at,
                    value: Value { at, kind: Kind::Bool(true) },
                }])),
            },
        }])),
    };
    let mut composed = None;
    let mut started = false;
    for (index, source, layer) in layers(target, documents).into_iter().flatten() {
        if let Some(patch) = &source.debian {
            if !started {
                composed = merge.merge(None, &bottom, Layer::Defaults, 0);
                started = true;
            }
            composed = merge.merge(composed, patch, layer, index);
        }
    }
    composed
}
