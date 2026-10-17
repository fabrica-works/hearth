//! Cache admission preserves the first record and never evicts (store.md, 6.8, 8.1).
use crate::{
    Store,
    limits::Limits,
    vocabulary::{Answer, CacheFile, Event, Limit, Output, Request, Step},
};
use hearth_lib::encode::Digest;
use skein_lib::{Env, Queue, Token};
#[derive(Debug)]
pub(super) struct Entering {
    arrival: Token,
    sha256: Digest,
    kept: bool,
    refused: bool,
}
pub(super) fn step(store: &mut Store, env: &Env<Limits>, event: Event, out: &mut Queue<Output>) {
    match event {
        Event::Arrived { arrival, sha256, size, origin, took } => {
            assert!(store.entering.is_none(), "one arrival");
            let present = store.scan.index.get(&sha256) == Some(&true);
            let refused = !store.usable || (!present && store.scan.index.len() >= env.limits.cache_files);
            let kept = !present && !refused;
            store.entering = Some(Entering { arrival, sha256, kept, refused });
            let request = if kept {
                store.scan.index.insert(sha256, true).expect("admitted cache file");
                Request::Keep { arrival, record: CacheFile { at: env.wall, origin, sha256, size, took } }
            } else {
                Request::Discard { arrival }
            };
            out.push(Output::Request(request));
        }
        Event::Done | Event::Failed => {
            let entering = store.entering.take().expect("arrival terminal");
            let failed = event == Event::Failed;
            if failed {
                if entering.kept {
                    store.scan.index.remove(&entering.sha256);
                }
                store.usable = false;
            }
            if failed {
                out.push(Output::Answer(Answer::Unusable {
                    step: if entering.kept { Step::Keep } else { Step::Remove },
                }));
            } else if entering.refused && store.usable {
                out.push(Output::Answer(Answer::Full { limit: Limit::CacheFiles, most: env.limits.cache_files }));
            }
            let answer = if entering.refused || failed {
                Answer::Refused { arrival: entering.arrival }
            } else {
                Answer::Cached { arrival: entering.arrival }
            };
            out.push(Output::Answer(answer));
        }
        Event::Find { .. }
        | Event::Resolution { .. }
        | Event::Unreadable
        | Event::Open
        | Event::Locked
        | Event::Busy
        | Event::Loaded { .. }
        | Event::Page { .. } => {
            unreachable!("arrival terminal")
        }
    }
}
