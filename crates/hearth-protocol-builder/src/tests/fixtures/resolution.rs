use crate::{render, tree};
use skein_lib::{Duration, Wall};
#[must_use]
pub fn resolution_specs() -> [render::parts::Spec; 3] {
    [
        None,
        Some(render::keys::Resolution::Needed { key: super::ABC }),
        Some(render::keys::Resolution::Known {
            key: super::ABC,
            keys: render::keys::Keys { packages: super::ABC, software: super::ABC, config: super::ABC },
        }),
    ]
    .map(|resolution| {
        let mut spec = super::spec();
        spec.resolution = boxed_resolution(resolution);
        spec
    })
}
#[must_use]
pub fn resolution_record(inputs: tree::resolution::Inputs) -> tree::resolution::Resolution {
    use tree::resolution::*;
    Resolution {
        key: super::ABC,
        digest: super::ABC,
        inputs,
        closure: Box::new([Package {
            name: b"apt".as_slice().into(),
            architecture: b"amd64".as_slice().into(),
            version: b"3.3.3".as_slice().into(),
            source: 0,
            filename: b"pool/apt.deb".as_slice().into(),
            size: 7,
            sha256: super::ABC,
        }]),
        sources: Box::new([Line {
            uri: b"http://example.org".as_slice().into(),
            suite: b"forky".as_slice().into(),
            components: Box::new([b"main".as_slice().into()]),
            indices: Box::new([Index { path: b"InRelease".as_slice().into(), size: 3, sha256: super::ABC }]),
        }]),
        made: Made {
            at: Wall::EPOCH,
            apt: b"3.3.3".as_slice().into(),
            mmdebstrap: b"1.5.7".as_slice().into(),
            costs: Costs { fetched: 7, took: Duration::from_secs(1) },
        },
    }
}

fn boxed_resolution(value: Option<render::keys::Resolution>) -> Option<Box<render::keys::Resolution>> {
    Some(Box::new(value?))
}

#[must_use]
pub fn resolution_faults() -> [render::errors::Resolution; 9] {
    use render::errors::Resolution;
    [
        Resolution::Unknown { package: b"apt".as_slice().into() },
        Resolution::Unsatisfiable { package: Some(b"curl".as_slice().into()) },
        Resolution::Unsatisfiable { package: None },
        Resolution::Absent { uri: b"http://one/".as_slice().into(), suite: b"forky".as_slice().into() },
        Resolution::Missing { package: b"apt".as_slice().into() },
        Resolution::Unmatched {
            name: b"apt".as_slice().into(),
            version: b"3.3.3".as_slice().into(),
            architecture: b"amd64".as_slice().into(),
        },
        Resolution::Disagree {
            name: b"apt".as_slice().into(),
            version: b"3.3.3".as_slice().into(),
            architecture: b"amd64".as_slice().into(),
        },
        Resolution::TooLarge { max: u64::from(tree::limits::SHIPPED.resolution.body) },
        Resolution::TooMany { max: super::CLOSURE_LIMIT },
    ]
}

#[must_use]
pub fn resolution_inputs() -> tree::resolution::Inputs {
    use hearth_lib::encode::Item;
    tree::resolution::Inputs {
        sources: Box::new([Item::Map(0)]),
        request: Box::new([Item::List(1), Item::Bytes(b"hello".as_slice().into())]),
        variant: Box::new([Item::Bytes(b"apt".as_slice().into())]),
        apt: Box::new([Item::Map(0)]),
    }
}
#[must_use]
pub fn writable_resolutions() -> [tree::resolution::Resolution; 2] {
    let first = resolution_record(resolution_inputs());
    let mut second = first.clone();
    second.made.at = Wall::from_nanos(77);
    second.made.costs.fetched = 9;
    [first, second]
}
