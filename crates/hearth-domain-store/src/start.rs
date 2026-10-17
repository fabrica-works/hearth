//! Lock and record transitions (store.md, 8.2, steps 1 and 2).
use crate::{
    Phase, Store, Timer,
    limits::Limits,
    vocabulary::{Answer, Event, Output, Request, StoreRecord, Why},
};
use skein_lib::{Env, Queue};
pub(super) fn step(store: &mut Store, env: &Env<Limits>, ev: Event, out: &mut Queue<Output>) {
    let request = match ev {
        Event::Open => {
            store.phase = Phase::Locking;
            Request::Lock
        }
        Event::Busy => {
            store.timers.arm(Timer::Retry, env.now.saturating_add(env.limits.retry)).expect("one retry");
            return;
        }
        Event::Locked => {
            store.timers.cancel(Timer::Retry);
            store.phase = Phase::Recording;
            Request::LoadRecord
        }
        Event::Loaded { record } => {
            if let Some(record) = record
                && record.layout == 1
            {
                if record.uid != store.uid {
                    out.push(Output::Answer(Answer::Unopened {
                        why: Why::Owner { recorded: record.uid, builder: store.uid },
                    }));
                    store.phase = Phase::Closed;
                    return;
                }
                crate::scan::begin(store, env.limits, out);
                return;
            }
            store.phase = Phase::Writing;
            Request::WriteRecord { record: StoreRecord { layout: 1, made: env.wall, range: None, uid: store.uid } }
        }
        Event::Done => {
            crate::scan::begin(store, env.limits, out);
            return;
        }
        Event::Arrived { .. } | Event::Page { .. } => unreachable!("scan has not begun"),
        Event::Failed => {
            let why = if store.phase == Phase::Locking { Why::Lock } else { Why::Record };
            store.timers.cancel(Timer::Retry);
            store.phase = Phase::Closed;
            out.push(Output::Answer(Answer::Unopened { why }));
            return;
        }
    };
    out.push(Output::Request(request));
}
