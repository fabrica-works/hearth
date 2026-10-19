//! Meta's changed fields (domain/render.md, 6.1).
use crate::diff::{MemberPair, MetaChange, NamePair, PlacementPair};
use crate::{
    form::{json::field, text::Lines},
    parts::meta::Placement,
};
use skein_json::writer::Encoder;
enum Value<'a> {
    Bytes(Option<&'a [u8]>),
    Placement(&'a Placement),
}
struct Field<'a> {
    name: &'static [u8],
    pair: Option<ValuePair<'a>>,
}
struct ValuePair<'a> {
    before: Value<'a>,
    after: Value<'a>,
}
impl MetaChange {
    fn fields(&self) -> [Field<'_>; 4] {
        [
            Field { name: b"base", pair: bytes(self.base.as_ref()) },
            Field { name: b"member", pair: member(self.member.as_ref()) },
            Field { name: b"environment", pair: bytes(self.environment.as_ref()) },
            Field { name: b"placement", pair: placement(self.placement.as_ref()) },
        ]
    }
    pub(crate) fn text(&self, lines: &mut Lines) {
        for Field { name, pair } in self.fields() {
            if let Some(ValuePair { before, after }) = pair {
                lines.put(b"  meta     ~ ");
                lines.put(name);
                lines.put(b"  ");
                before.text(lines);
                lines.put(" → ".as_bytes());
                after.text(lines);
                lines.put(b"\n");
            }
        }
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        json.object_start();
        field(json, b"kind", b"meta");
        for Field { name, pair } in self.fields() {
            if let Some(ValuePair { before, after }) = pair {
                json.key(name);
                json.object_start();
                json.key(b"before");
                before.json(json);
                json.key(b"after");
                after.json(json);
                json.object_end();
            }
        }
        json.object_end();
    }
}
fn bytes(pair: Option<&NamePair>) -> Option<ValuePair<'_>> {
    let pair = pair?;
    Some(ValuePair { before: Value::Bytes(Some(&pair.before)), after: Value::Bytes(Some(&pair.after)) })
}
impl Value<'_> {
    fn text(&self, lines: &mut Lines) {
        match self {
            Self::Bytes(Some(bytes)) => lines.put(bytes),
            Self::Bytes(None) => lines.put(b"none"),
            Self::Placement(placement) => placement_text(placement, lines),
        }
    }
    fn json(&self, json: &mut Encoder) {
        match self {
            Self::Bytes(Some(bytes)) => json.string(bytes),
            Self::Bytes(None) => json.null(),
            Self::Placement(placement) => placement_json(placement, json),
        }
    }
}

fn member(pair: Option<&MemberPair>) -> Option<ValuePair<'_>> {
    let p = pair?;
    Some(ValuePair { before: Value::Bytes(p.before.as_deref()), after: Value::Bytes(p.after.as_deref()) })
}
fn placement(pair: Option<&PlacementPair>) -> Option<ValuePair<'_>> {
    let p = pair?;
    Some(ValuePair { before: Value::Placement(&p.before), after: Value::Placement(&p.after) })
}

fn placement_text(p: &Placement, lines: &mut Lines) {
    use crate::parts::meta::{Firmware, PlacementKind};
    lines.put(match p.kind {
        PlacementKind::Metal => b"metal",
        PlacementKind::Vm => b"vm",
        PlacementKind::Container => b"container",
    });
    if let Some(firmware) = p.firmware {
        lines.put(b" ");
        lines.put(match firmware {
            Firmware::Efi => b"efi",
            Firmware::Bios => b"bios",
        });
    }
    if let Some(hardware) = &p.hardware {
        lines.put(b" ");
        lines.put(hardware);
    }
    if let Some(on) = &p.on {
        lines.put(b" on ");
        lines.put(on);
    }
}
fn placement_json(p: &Placement, json: &mut Encoder) {
    use crate::parts::meta::{Firmware, PlacementKind};
    json.object_start();
    if let Some(firmware) = p.firmware {
        field(
            json,
            b"firmware",
            match firmware {
                Firmware::Efi => b"efi",
                Firmware::Bios => b"bios",
            },
        );
    }
    if let Some(hardware) = &p.hardware {
        field(json, b"hardware", hardware);
    }
    field(
        json,
        b"kind",
        match p.kind {
            PlacementKind::Metal => b"metal",
            PlacementKind::Vm => b"vm",
            PlacementKind::Container => b"container",
        },
    );
    if let Some(on) = &p.on {
        field(json, b"on", on);
    }
    json.object_end();
}
