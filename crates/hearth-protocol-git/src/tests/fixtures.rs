use crate::{
    limits::Limits,
    listing::{self, Event, Listing, Request},
};
use skein_lib::{
    Env, Queue, Time, Wall, bytes,
    stream::{Down, Held},
};
pub fn listed(stdout: &[u8], limits: &Limits) -> Event {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut machine = Listing::new(limits);
    let mut held = Held::new(bytes::copy_of(stdout));
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    listing::down(&mut machine, &env, Request::List, &mut above, &mut below);
    for _ in 0..stdout.len().saturating_add(4) {
        if let Some(answer) = above.pop() {
            return answer;
        }
        match below.pop().expect("one outstanding read") {
            Down::Demand { read, room: 0 } => {
                let up = held.answer(read).expect("held answers active demand");
                listing::up(&mut machine, &env, up, &mut above, &mut below);
            }
            Down::Demand { .. } | Down::Send(_) | Down::Finish => unreachable!("read only"),
        }
    }
    unreachable!("bounded records settle")
}
