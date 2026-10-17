use crate::{
    limits,
    vector::{Apt, Component, Name, Simulate, Sources, Timestamp, Variant},
};
use alloc::boxed::Box;
use skein_lib::List;
pub fn name(bytes: &[u8]) -> Name {
    Name::new(bytes, &limits::SHIPPED).unwrap()
}
pub fn forky_main() -> Sources {
    Sources {
        suite: name(b"forky"),
        architecture: name(b"amd64"),
        components: Box::from([Component::Main]),
        archive: Timestamp::new(b"20261001T030405Z").unwrap(),
        updates: false,
        security: None,
    }
}
pub fn simulate(names: &[&[u8]]) -> Simulate {
    let mut request = List::with_capacity(limits::SHIPPED.request);
    for n in names {
        request.push(name(n)).unwrap();
    }
    Simulate {
        sources: forky_main(),
        request: request.into_boxed(),
        variant: Variant::Apt,
        apt: Apt { install_recommends: false },
    }
}
