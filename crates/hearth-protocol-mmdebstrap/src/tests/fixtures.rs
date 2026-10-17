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

pub fn opened() -> (
    skein_lib::Env<limits::Limits>,
    crate::output::Reader,
    skein_lib::Queue<crate::output::Seen>,
    skein_lib::Queue<skein_lib::stream::Down>,
) {
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: limits::SHIPPED };
    let mut reader = crate::output::Reader::new(&env.limits);
    let mut below = skein_lib::Queue::with_capacity(1);
    crate::output::open(&mut reader, &env, &mut below);
    assert!(below.pop().is_some(), "open demands a scan");
    (env, reader, skein_lib::Queue::with_capacity(1), below)
}
