mod fixtures;
use crate::{
    Builder,
    render::vocabulary::{Answer as RenderAnswer, Output},
    step,
    vocabulary::{Answer, Event, Request},
};
use skein_lib::{Queue, ReplyTo, Token, bytes};

#[test]
fn a_render_call_reaches_render() {
    let mut builder = Builder::new(&fixtures::LIMITS, fixtures::MIN_UID);
    let mut out = Queue::with_capacity(1);
    step(
        &mut builder,
        &fixtures::env(),
        Event::Call { reply_to: ReplyTo::new(Token::new(1)), call: fixtures::call() },
        &mut out,
    );
    assert_eq!(
        out.pop(),
        Some(Request::Store(crate::store::vocabulary::Request::Lock)),
        "the root opens its store first"
    );
}
#[test]
fn renders_spec_reaches_the_client() {
    let mut builder = Builder::new(&fixtures::LIMITS, fixtures::MIN_UID);
    let mut out = Queue::with_capacity(1);
    builder.output.push(Output::Answer(RenderAnswer::Spec(fixtures::spec())));
    builder.forward(&fixtures::env(), &mut out);
    let Some(Request::Answer(Answer::Spec(spec))) = out.pop() else {
        panic!("client spec");
    };
    assert_eq!(&*spec.meta.target, b"x1-prod", "render's target");
    assert_eq!(spec.digest, hearth_lib::encode::Digest([2; 32]), "render's digest");
}
#[test]
#[should_panic(expected = "one builder call at a time")]
fn a_second_call_while_one_is_held_panics() {
    let mut builder = Builder::new(&fixtures::LIMITS, fixtures::MIN_UID);
    let mut out = Queue::with_capacity(1);
    step(
        &mut builder,
        &fixtures::env(),
        Event::Call { reply_to: ReplyTo::new(Token::new(1)), call: fixtures::call() },
        &mut out,
    );
    assert!(out.pop().is_some(), "first call starts");
    step(
        &mut builder,
        &fixtures::env(),
        Event::Call { reply_to: ReplyTo::new(Token::new(2)), call: fixtures::call() },
        &mut out,
    );
}

#[test]
fn the_call_reaches_render_only_once_the_store_is_open() {
    use crate::store::vocabulary::Event as StoreEvent;
    let mut builder = Builder::new(&fixtures::LIMITS, fixtures::MIN_UID);
    let mut out = Queue::with_capacity(1);
    step(
        &mut builder,
        &fixtures::env(),
        Event::Call { reply_to: ReplyTo::new(Token::new(1)), call: fixtures::call() },
        &mut out,
    );
    assert_eq!(out.pop(), Some(Request::Store(crate::store::vocabulary::Request::Lock)));
    step(&mut builder, &fixtures::env(), Event::Store(crate::store::vocabulary::Event::Locked), &mut out);
    assert_eq!(out.pop(), Some(Request::Store(crate::store::vocabulary::Request::LoadRecord)));
    let record = Some(fixtures::store_record());
    step(&mut builder, &fixtures::env(), Event::Store(StoreEvent::Loaded { record }), &mut out);
    for _ in 0_u8..5 {
        drop(out.pop());
        step(
            &mut builder,
            &fixtures::env(),
            Event::Store(crate::store::vocabulary::Event::Page { entries: Box::new([]), last: true }),
            &mut out,
        );
    }
    drop(out.pop());
    step(&mut builder, &fixtures::env(), Event::Store(StoreEvent::Done), &mut out);
    assert_eq!(
        out.pop(),
        Some(Request::Render(crate::render::vocabulary::Request::List {
            request: Token::new(1),
            repository: bytes::copy_of(b"/repo"),
            revision: bytes::copy_of(b"HEAD")
        }))
    );
}
const UNOPENED: &[crate::store::vocabulary::Why] = &[
    crate::store::vocabulary::Why::Lock,
    crate::store::vocabulary::Why::Record,
    crate::store::vocabulary::Why::Owner { recorded: fixtures::MIN_UID + 1, builder: fixtures::MIN_UID },
];
#[test]
fn an_unopened_store_ends_the_call_before_render() {
    for &why in UNOPENED {
        let mut builder = Builder::new(&fixtures::LIMITS, fixtures::MIN_UID);
        let mut out = Queue::with_capacity(1);
        step(
            &mut builder,
            &fixtures::env(),
            Event::Call { reply_to: ReplyTo::new(Token::new(1)), call: fixtures::call() },
            &mut out,
        );
        drop(out.pop());
        builder
            .store_output
            .push(crate::store::vocabulary::Output::Answer(crate::store::vocabulary::Answer::Unopened { why }));
        crate::store_forward(&mut builder, &fixtures::env(), &mut out);
        assert_eq!(out.pop(), Some(Request::Unopened { to: ReplyTo::new(Token::new(1)), why }));
        assert!(builder.is_empty());
        assert!(!builder.work_pending());
    }
}

#[test]
fn a_refused_blob_fails_its_read_at_its_last_blob() {
    use crate::{
        render::vocabulary::{Event as RenderEvent, Outcome},
        store::vocabulary as store,
    };
    let mut builder = Builder::new(&fixtures::LIMITS, fixtures::MIN_UID);
    builder.reply = Some(ReplyTo::new(Token::new(1)));
    let mut out = Queue::with_capacity(1);
    step(
        &mut builder,
        &fixtures::env(),
        Event::Render(RenderEvent::Render {
            resolve: false,
            repository: bytes::copy_of(b"/repo"),
            revision: bytes::copy_of(b"HEAD"),
            targets: Box::new([]),
        }),
        &mut out,
    );
    drop(out.pop());
    builder.remaining = 2;
    fixtures::arrival_answer(&mut builder, false, store::Answer::Refused { arrival: Token::new(10) });
    crate::store_forward(&mut builder, &fixtures::env(), &mut out);
    assert_eq!(out.pop(), None, "a refused read is drained before failing");
    fixtures::arrival_answer(&mut builder, true, store::Answer::Cached { arrival: Token::new(10) });
    crate::store_forward(&mut builder, &fixtures::env(), &mut out);
    assert_eq!(out.pop(), None);
    crate::resume(&mut builder, &fixtures::env(), &mut out);
    assert_eq!(
        out.pop(),
        Some(Request::Reply { to: ReplyTo::new(Token::new(1)), outcome: Outcome::TryAgain, diff: None, left_out: 0 })
    );
}

#[test]
fn a_full_store_is_answered_once_before_renders_answers() {
    use crate::{
        render::errors::{Error, Kind, StoreError, StoreFault, StoreLimit},
        store::vocabulary::{Answer as StoreAnswer, Limit, Output as StoreOutput},
    };
    let mut builder = Builder::new(&fixtures::LIMITS, fixtures::MIN_UID);
    builder.call = Some(fixtures::call());
    let mut out = Queue::with_capacity(2);
    builder.store_output.push(StoreOutput::Answer(StoreAnswer::Full { limit: Limit::CacheFiles, most: 2 }));
    crate::store_forward(&mut builder, &fixtures::env(), &mut out);
    assert_eq!(
        out.pop(),
        Some(Request::Answer(Answer::Error(Error {
            side: None,
            kind: Kind::Store(StoreError {
                store: Box::new([]),
                fault: StoreFault::Full { limit: StoreLimit::CacheFiles, most: 2 }
            }),
            class: crate::render::vocabulary::Class::Operational,
            at: None,
            also: None,
            target: None,
            task: None,
        })))
    );
    assert_eq!(
        out.pop(),
        Some(Request::Render(crate::render::vocabulary::Request::List {
            request: Token::new(1),
            repository: bytes::copy_of(b"/repo"),
            revision: bytes::copy_of(b"HEAD")
        }))
    );
    builder.store_output.push(StoreOutput::Answer(StoreAnswer::Full { limit: Limit::CacheFiles, most: 2 }));
    crate::store_forward(&mut builder, &fixtures::env(), &mut out);
    assert_eq!(out.pop(), None);
}

#[test]
fn a_store_error_makes_the_call_operational_even_without_an_arrival() {
    let mut builder = Builder::new(&fixtures::LIMITS, fixtures::MIN_UID);
    builder.store_error = true;
    builder.reply = Some(ReplyTo::new(Token::new(0)));
    builder.output.push(Output::Answer(RenderAnswer::Rendered {
        outcome: crate::render::vocabulary::Outcome::Done,
        diff: None,
        left_out: 0,
    }));
    let mut out = Queue::with_capacity(2);
    builder.forward(&fixtures::env(), &mut out);
    assert_eq!(
        out.pop(),
        Some(Request::Reply {
            to: ReplyTo::new(Token::new(0)),
            outcome: crate::render::vocabulary::Outcome::TryAgain,
            diff: None,
            left_out: 0
        })
    );
}

const RENDERED_PATHS: &[&[u8]] = &[b"tasks/motd/tree/etc/hostname.j2", b"tasks/motd/tree/etc/issue.j2"];
#[test]
fn a_produced_file_enters_the_store_with_its_rendered_origin() {
    use crate::store::vocabulary::{CacheFile, Origin, Request as StoreRequest};
    for (row, path) in RENDERED_PATHS.iter().enumerate() {
        let mut builder = fixtures::opened_builder();
        let mut out = Queue::with_capacity(2);
        step(
            &mut builder,
            &fixtures::env(),
            Event::Produced {
                request: Token::new(4),
                path: bytes::copy_of(path),
                arrival: Token::new(10),
                sha256: hearth_lib::encode::Digest([1; 32]),
                size: 8,
                took: skein_lib::Duration::from_millis(1),
            },
            &mut out,
        );
        assert_eq!(
            out.pop(),
            Some(Request::Store(StoreRequest::Keep {
                arrival: Token::new(10),
                record: CacheFile {
                    at: skein_lib::Wall::EPOCH,
                    origin: Origin::Rendered { path: bytes::copy_of(path) },
                    sha256: hearth_lib::encode::Digest([1; 32]),
                    size: 8,
                    took: skein_lib::Duration::from_millis(1),
                },
            })),
            "row {row}"
        );
    }
}

#[test]
fn a_find_reaches_the_store_and_its_answer_render() {
    let mut builder = fixtures::indexed_builder();
    let mut out = Queue::with_capacity(1);
    builder.output.push(Output::Request(crate::render::vocabulary::Request::Find {
        request: Token::new(5),
        key: fixtures::FIND_KEY,
    }));
    builder.forward(&fixtures::env(), &mut out);
    assert_eq!(out.pop(), Some(Request::Store(crate::store::vocabulary::Request::Load { key: fixtures::FIND_KEY })));
}
#[test]
fn a_found_resolution_answers_render_with_the_stored_digest() {
    let crate::render::vocabulary::Event::Found { resolution, .. } =
        crate::resolution::answer(Token::new(5), crate::resolution::Settled::Found(fixtures::store_resolution()))
    else {
        panic!("found record")
    };
    assert_eq!(resolution.digest, fixtures::EMPTY_CLOSURE);
}

#[test]
fn a_declined_record_answers_the_stores_error_once_and_leaves_the_key_needed() {
    let (mut builder, mut out) = fixtures::record_builder();
    drop(out.pop());
    step(&mut builder, &fixtures::env(), Event::Store(crate::store::vocabulary::Event::Failed), &mut out);
    let requests = fixtures::finish(&mut builder, &mut out);
    assert_eq!(fixtures::error_count(&requests), 1);
    let mut needed = 0_u32;
    for request in requests {
        if let Request::Answer(Answer::Spec(spec)) = request {
            let Some(crate::render::keys::Resolution::Needed { .. }) = spec.resolution.as_deref() else {
                panic!("needed key")
            };
            needed = needed.saturating_add(1);
        }
    }
    assert_eq!(needed, 1);
}
#[test]
fn a_resolve_goes_down_only_when_the_store_has_room() {
    let env = skein_lib::Env { limits: fixtures::NO_RESOLUTION_ROOM, ..fixtures::env() };
    let (mut builder, mut out) = fixtures::resolving_builder(&env);
    let requests = fixtures::finish(&mut builder, &mut out);
    assert_eq!(fixtures::error_count(&requests), 1);
    let mut resolves = 0_u32;
    for request in requests {
        if let Request::Render(crate::render::vocabulary::Request::Resolve { .. }) = request {
            resolves = resolves.saturating_add(1);
        }
    }
    assert_eq!(resolves, 0);
}

#[test]
fn each_resolution_field_translates_and_back() {
    for (row, input) in fixtures::resolutions().into_iter().enumerate() {
        let back = crate::resolution::stored(crate::resolution::found(input.clone()));
        assert_eq!(back, input, "row {row}: resolution record");
    }
}

#[test]
fn a_refused_index_fails_its_resolve_and_nothing_is_recorded() {
    let (mut builder, mut out, _) = fixtures::resolving_request();
    step(&mut builder, &fixtures::env(), fixtures::index(), &mut out);
    let Some(Request::Store(crate::store::vocabulary::Request::Keep { .. })) = out.pop() else { panic!("keep index") };
    step(&mut builder, &fixtures::env(), Event::Store(crate::store::vocabulary::Event::Failed), &mut out);
    let requests = fixtures::finish_resolution(&mut builder, &mut out);
    assert_eq!(fixtures::error_count(&requests), 1);
    assert_eq!(fixtures::needed_count(&requests), 1);
    let mut writes = 0_u32;
    for request in requests {
        if let Request::Store(crate::store::vocabulary::Request::Write { .. }) = request {
            writes = writes.saturating_add(1);
        }
    }
    assert_eq!(writes, 0);
}

#[test]
fn a_resolve_says_it_starts_and_ends() {
    use crate::vocabulary::Progress;
    let (mut builder, mut out) = fixtures::resolving_progress();
    crate::store_step(&mut builder, &fixtures::env(), crate::store::vocabulary::Event::Room, &mut out);
    assert_eq!(out.pop(), Some(Request::Progress(Progress::Resolving { key: fixtures::FIND_KEY })));
    drop(out.pop());
    drop(crate::resolution::progress(&mut builder, fixtures::resolved_render(), &mut out));
    assert_eq!(
        out.pop(),
        Some(Request::Progress(Progress::Resolved {
            key: fixtures::FIND_KEY,
            took: skein_lib::Duration::from_millis(7)
        }))
    );
}

#[test]
fn an_index_enters_with_the_key_its_resolve_holds() {
    let (mut builder, mut out) = fixtures::resolving_progress();
    crate::store_step(&mut builder, &fixtures::env(), crate::store::vocabulary::Event::Room, &mut out);
    drop(out.pop());
    drop(out.pop());
    step(&mut builder, &fixtures::env(), fixtures::index(), &mut out);
    let Some(Request::Store(crate::store::vocabulary::Request::Keep { record, .. })) = out.pop() else {
        panic!("keep index")
    };
    assert_eq!(record.origin, crate::store::vocabulary::Origin::Index { resolution: fixtures::FIND_KEY });
}
