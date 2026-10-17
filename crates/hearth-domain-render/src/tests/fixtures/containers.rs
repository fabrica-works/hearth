use super::{composition::at, with_document};
use crate::{compose, errors, limits, revision};
use skein_lib::bytes::copy_of;

pub struct Fleet {
    pub documents: revision::Documents,
    pub targets: Box<[compose::Target]>,
}
pub fn fleet() -> Fleet {
    let mut documents = revision::Documents::new(&[]);
    for path in [b"envs/prod.json".as_slice(), b"envs/lab.json"] {
        documents = with_document(&documents, path, document(None));
    }
    for (path, on) in [
        (b"hosts/rhi/rhi-prod.json".as_slice(), None),
        (b"hosts/rhi/rhi-lab.json", None),
        (b"hosts/chat/chat-prod.json", Some(b"rhi-prod".as_slice())),
        (b"hosts/mail/mail-prod.json", Some(b"rhi-prod")),
        (b"hosts/mail/mail-lab.json", Some(b"rhi-lab")),
        (b"hosts/empty/empty-prod.json", None),
    ] {
        documents = with_document(&documents, path, document(Some(placement(on))));
    }
    discovered(documents)
}
pub fn discovered(documents: revision::Documents) -> Fleet {
    let mut errors = errors::Collector::new(&limits::SHIPPED);
    let targets = compose::targets(&documents, &limits::SHIPPED, &mut errors).expect("fixture targets").targets;
    Fleet { documents, targets }
}
pub fn document(placement: Option<revision::Placement>) -> revision::Document {
    revision::Document { at: at(1, 1), members: None, layer: revision::Layer { placement, tasks: Box::new([]) } }
}
pub fn placement(on: Option<&[u8]>) -> revision::Placement {
    let mut link = None;
    if let Some(target) = on {
        link = Some(revision::On { target: copy_of(target), at: at(1, 43) });
    }
    revision::Placement {
        at: at(1, 2),
        kind: if on.is_some() { revision::Runs::Container } else { revision::Runs::Vm },
        firmware: None,
        hardware: None,
        on: link,
        mutable: false,
    }
}
pub fn target<'a>(fleet: &'a Fleet, name: &[u8]) -> &'a compose::Target {
    for target in &fleet.targets {
        if target.name.as_ref() == name {
            return target;
        }
    }
    panic!("fixture host")
}
pub fn on(fleet: &Fleet, name: &[u8]) -> Result<Box<[compose::Container]>, Box<errors::Error>> {
    compose::containers(target(fleet, name), &fleet.targets, &fleet.documents)
}
pub fn names(containers: &[compose::Container]) -> Box<[&[u8]]> {
    let mut names = skein_lib::List::with_capacity(u32::try_from(containers.len()).expect("fixture count"));
    for container in containers {
        names.push(container.name.as_ref()).expect("fixture count");
    }
    names.into_boxed()
}
pub fn root(fleet: &Fleet, name: &[u8]) -> crate::evaluate::Value {
    let containers = on(fleet, name).expect("valid containers");
    let root = crate::evaluate::context::target(&super::meta(), &containers);
    let crate::evaluate::Value::Map(entries) = root else { panic!("target map") };
    for entry in entries {
        if entry.key.as_ref() == b"containers" {
            return entry.value.expect("map");
        }
    }
    panic!("containers key")
}

pub const RHI_PROD_CONTAINERS: &[&[u8]] = &[b"chat-prod", b"mail-prod"];
