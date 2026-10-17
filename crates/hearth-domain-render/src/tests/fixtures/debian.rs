use super::{
    bytes,
    composition::{at, map, string},
};
use crate::{
    compose::{Composed, Composite, Layer, Target},
    revision,
};
pub fn written_debian() -> revision::Value {
    map(at(1, 1), &super::forms::debian_fields())
}
pub fn debian_layers(values: [Option<&str>; 6]) -> (revision::Documents, Target) {
    let [fleet, base, member, environment, base_env, base_env_member] = values;
    let documents = super::member_layers([None; 5]);
    let empty = revision::Document {
        at: at(1, 1),
        members: None,
        layer: revision::Layer { debian: None, placement: None, tasks: Box::new([]) },
    };
    let mut documents = super::with_document(&documents, b"fleet.json", empty);
    for (path, value, member) in [
        (b"fleet.json".as_slice(), fleet, false),
        (b"hosts/stoa/stoa.json", base, false),
        (b"hosts/stoa/stoa.json", member, true),
        (b"envs/lab.json", environment, false),
        (b"hosts/stoa/stoa-lab.json", base_env, false),
        (b"hosts/stoa/stoa-lab.json", base_env_member, true),
    ] {
        let index = documents.find(path).expect("fixture path");
        let mut document = documents.get(index).expect("fixture document").clone();
        let debian = suite_patch(value);
        if member {
            super::entries::only_mut(&mut document.members.as_mut().expect("member").each)
                .layer
                .as_mut()
                .expect("layer")
                .debian = debian;
        } else {
            document.layer.debian = debian;
        }
        documents.set(&revision::Oid(bytes::copy_of(path)), document);
    }
    let target = Target {
        name: bytes::copy_of(b"stoa-1-lab"),
        base: bytes::copy_of(b"stoa"),
        environment: bytes::copy_of(b"lab"),
        member: Some(bytes::copy_of(b"1")),
        fleet: documents.find(b"fleet.json"),
        base_document: documents.find(b"hosts/stoa/stoa.json"),
        environment_document: documents.find(b"envs/lab.json").expect("environment"),
        base_env: documents.find(b"hosts/stoa/stoa-lab.json").expect("base-env"),
    };
    (documents, target)
}
fn composition_null() -> revision::Value {
    super::composition::null(at(1, 1))
}
pub fn suite(composed: &Composed) -> Option<(&[u8], Layer)> {
    if let Composite::Map(pairs) = &composed.kind {
        for pair in pairs {
            if pair.key.as_ref() == b"suite"
                && let Composite::String(value) = &pair.value.kind
            {
                return Some((value, pair.value.from.layer));
            }
        }
    }
    None
}

pub fn checked_debian() -> crate::evaluate::Value {
    super::forms::debian(&[]).0.expect("valid fixed schema")
}
pub fn debian_guarded_tasks() -> (crate::call::Side, crate::schema::Validated) {
    let (mut side, checked) = super::guarded_tasks();
    let condition = super::condition([
        super::name(0, b"debian", crate::evaluate::Missing::Error),
        super::key(0, b"suite", crate::evaluate::Missing::Error),
        super::op(0, crate::evaluate::Op::String(bytes::copy_of(b"trixie"))),
        super::op(0, crate::evaluate::Op::Equal),
    ]);
    let declared = super::requires([super::guard(b"suite", at(1, 1), condition, b"suite missing")]);
    side.documents = super::declarations(&[(b"core", Some(declared.clone())), (b"motd", Some(declared))]);
    (side, checked)
}
pub fn debian_entry() -> revision::declaration::Declaration {
    use crate::evaluate::Missing;
    let key = super::parts([super::entry_output([
        super::name(0, b"debian", Missing::Error),
        super::key(0, b"suite", Missing::Error),
    ])]);
    let when = super::condition([super::name(0, b"debian", Missing::Error), super::key(0, b"suite", Missing::Error)]);
    super::entries([super::entry(key, None, Some(when))])
}

fn suite_patch(value: Option<&str>) -> Option<revision::Value> {
    let suite = value?;
    Some(if suite == "null" {
        composition_null()
    } else {
        map(at(1, 1), &[(b"suite", at(1, 2), string(at(1, 10), suite.as_bytes()))])
    })
}
