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

pub fn batched(stdout: &[u8], requests: &[crate::batch::Request], limits: &Limits) -> Box<[crate::batch::Event]> {
    use crate::batch::{self, Batch, Event, Request};
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut machine = Batch::START;
    let mut held = Held::new(bytes::copy_of(stdout));
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    let mut events = skein_lib::List::with_capacity(
        u32::try_from(stdout.len().saturating_add(requests.len())).expect("test input small"),
    );
    let mut next = requests.iter();
    let Some(request) = next.next() else {
        return events.into_boxed();
    };
    batch::down(&mut machine, &env, *request, &mut above, &mut below);
    for _ in 0..stdout.len().saturating_mul(3).saturating_add(20) {
        if let Some(event) = above.pop() {
            let request = match event {
                Event::Piece(_) => Some(Request::Next),
                Event::Whole { .. } | Event::Missing { .. } => next.next().copied(),
                Event::Failed(_) | Event::Closed => None,
            };
            events.push(event).expect("one event per byte or request");
            let Some(request) = request else {
                return events.into_boxed();
            };
            batch::down(&mut machine, &env, request, &mut above, &mut below);
        }
        match below.pop().expect("one outstanding read") {
            Down::Demand { read, room: 0 } => {
                let event = held.answer(read).expect("held answers read");
                batch::up(&mut machine, &env, event, &mut above, &mut below);
            }
            Down::Demand { .. } | Down::Send(_) | Down::Finish => unreachable!("read only"),
        }
    }
    unreachable!("bounded batch settles")
}

pub fn hunked(stdout: &[u8], limits: &Limits) -> crate::hunks::Event {
    use crate::hunks::{self, Diff, Request};
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut machine = Diff::new(limits);
    let mut held = Held::new(bytes::copy_of(stdout));
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    hunks::down(&mut machine, &env, Request::Read, &mut above, &mut below);
    for _ in 0..stdout.len().saturating_mul(3).saturating_add(20) {
        if let Some(event) = above.pop() {
            return event;
        }
        match below.pop().expect("outstanding demand") {
            Down::Demand { read, room: 0 } => {
                let event = held.answer(read).expect("held answers demand");
                hunks::up(&mut machine, &env, event, &mut above, &mut below);
            }
            Down::Demand { .. } | Down::Send(_) | Down::Finish => unreachable!("read only"),
        }
    }
    unreachable!("bounded diff settles")
}
