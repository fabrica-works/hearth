//! Both neighbours of one decoder, with reproducible delays and cuts.
use crate::reference;
use hearth_protocol_config::{
    Error, Fault,
    json::{self, Decoder, Event, Request, Value, Waiting},
    limits::{Limits, SHIPPED},
};
use skein_lib::{
    Env, Queue, Rng, Time, Wall,
    stream::{Down, Fault as StreamFault, Read, Up},
};
use skein_world::stream::Wire;
#[derive(Clone, Debug)]
pub struct Settings {
    pub limits: Limits,
    pub cut: Option<usize>,
    pub failure: Option<(u64, StreamFault)>,
    pub close: Option<Waiting>,
    pub eagerness: u32,
    pub stall: u64,
}
impl Settings {
    #[must_use]
    pub fn calm(rng: &mut Rng, limits: Limits) -> Settings {
        Settings {
            limits,
            cut: None,
            failure: None,
            close: None,
            eagerness: u32::try_from(rng.between(100, 1000)).expect("bounded"),
            stall: rng.below(32),
        }
    }
    #[must_use]
    pub fn chaotic(rng: &mut Rng, limits: Limits, length: usize) -> Settings {
        let mut settings = Self::calm(rng, limits);
        if rng.chance(150) {
            settings.cut = Some(usize::try_from(rng.below(length as u64 + 1)).expect("draw fits usize"));
        }
        if rng.chance(200) {
            settings.failure = Some((rng.below(20), StreamFault::Reset));
        }
        if rng.chance(200) {
            settings.close = Some(
                [Waiting::Decode, Waiting::Bytes, Waiting::Close]
                    [usize::try_from(rng.below(3)).expect("draw fits usize")],
            );
        }
        settings
    }
    #[must_use]
    pub fn sent<'a>(&self, bytes: &'a [u8]) -> &'a [u8] {
        &bytes[..self.cut.unwrap_or(bytes.len()).min(bytes.len())]
    }
}
#[must_use]
pub fn limits(rng: &mut Rng) -> Limits {
    if rng.chance(350) {
        Limits {
            document: u32::try_from(rng.below(200)).expect("draw fits u32"),
            depth: u32::try_from(rng.below(5)).expect("draw fits u32"),
            keys: u32::try_from(rng.below(5)).expect("draw fits u32"),
            settings: u32::try_from(rng.below(5)).expect("draw fits u32"),
        }
    } else {
        Limits { document: 4096, depth: 16, keys: 32, settings: 16 }
    }
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Outcome {
    Decoded,
    Failed(Error),
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Run {
    pub outcome: Option<Outcome>,
    pub value: Option<Value>,
    pub closed_at: Waiting,
    pub failed: Option<StreamFault>,
    pub early_end: bool,
    pub delivered: usize,
}
#[must_use]
pub fn check(document: &[u8], settings: &Settings, seed: u64) -> Run {
    let run = run(document, settings, seed);
    let expected = reference::decode(settings.sent(document), &settings.limits);
    match &run.outcome {
        Some(Outcome::Decoded) => assert_eq!(
            run.value.as_ref(),
            Some(&expected.expect("reference accepts")),
            "seed {seed}, {settings:?}, {document:?}"
        ),
        Some(Outcome::Failed(error)) => match &error.fault {
            Fault::Stream(fault) => {
                assert_eq!(run.failed, Some(*fault));
                assert_eq!(error.at, reference::position(settings.sent(document), run.delivered));
            }
            Fault::Json(_) => assert!(
                matches!(expected, Err(Error { fault: Fault::Json(_), .. })),
                "seed {seed}: {error:?} vs {expected:?}; {document:?}"
            ),
            Fault::NotObject
            | Fault::Duplicate { .. }
            | Fault::NotInteger
            | Fault::Range
            | Fault::Nul
            | Fault::Keys
            | Fault::Note
            | Fault::Shape(_)
            | Fault::Declaration(_) => {
                assert_eq!(Err(error.clone()), expected, "seed {seed}, {settings:?}, {document:?}");
            }
        },
        None => assert!(settings.close.is_some(), "only close may interrupt Decode"),
    }
    run
}
#[must_use]
pub fn run(document: &[u8], settings: &Settings, seed: u64) -> Run {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: settings.limits };
    let mut rng = Rng::new(seed);
    let mut decoder = Decoder::new(&env.limits);
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    let mut wire = Wire::new(seed);
    wire.write(settings.sent(document));
    wire.eof = true;
    let mut run =
        Run { outcome: None, value: None, closed_at: Waiting::Nothing, failed: None, early_end: false, delivered: 0 };
    let mut requested = false;
    let mut closing = false;
    let mut pending = false;
    for tick in 0..document.len() as u64 * 100 + 10_000 {
        if tick < settings.stall {
            continue;
        }
        if let Some(event) = above.pop() {
            match event {
                Event::Decoded(value) => {
                    assert!(requested && run.outcome.is_none());
                    run.value = Some(value);
                    run.outcome = Some(Outcome::Decoded);
                }
                Event::Failed(error) => {
                    assert!(requested && run.outcome.is_none());
                    run.outcome = Some(Outcome::Failed(error));
                }
                Event::Closed => {
                    assert!(closing);
                    assert_eq!(decoder.waiting(), Waiting::Nothing);
                    assert!(above.is_empty());
                    assert!(below.is_empty());
                    json::up(&mut decoder, &env, Up::End, &mut above, &mut below);
                    assert!(above.is_empty() && below.is_empty(), "Closed is once and last");
                    return run;
                }
            }
        }
        let waiting = decoder.waiting();
        if !closing && (settings.close == Some(waiting) || waiting == Waiting::Close) && rng.chance(settings.eagerness)
        {
            run.closed_at = waiting;
            closing = true;
            json::down(&mut decoder, &env, Request::Close, &mut above, &mut below);
        } else if waiting == Waiting::Decode && !requested && rng.chance(settings.eagerness) {
            requested = true;
            json::down(&mut decoder, &env, Request::Decode, &mut above, &mut below);
        } else if waiting == Waiting::Bytes && !closing {
            if let Some((at, fault)) = settings.failure
                && tick >= at
                && run.failed.is_none()
            {
                assert!(pending, "a read to fail");
                // A failure fulfils the wire's outstanding demand, as Bytes would.
                wire.take(Down::Demand { read: Read::Nothing, room: 0 });
                pending = false;
                run.failed = Some(fault);
                json::up(&mut decoder, &env, Up::Failed(fault), &mut above, &mut below);
            } else if let Some(event) = wire.answer() {
                assert!(pending);
                pending = false;
                match &event {
                    Up::Bytes(bytes) => run.delivered += bytes.len(),
                    Up::End => run.early_end = settings.sent(document).len() < document.len(),
                    Up::Failed(_) | Up::Room => panic!("the wire answers reads with bytes or end"),
                }
                json::up(&mut decoder, &env, event, &mut above, &mut below);
            }
        }
        assert!(above.len() <= 1 && below.len() <= 1, "MAX_OUT");
        if let Some(down) = below.pop() {
            match &down {
                Down::Demand { read: Read::Nothing, room: 0 } => {
                    assert!(closing && pending);
                    pending = false;
                }
                Down::Demand { read, room } => {
                    assert_eq!(*room, 0);
                    assert!(!pending);
                    match read {
                        Read::Fill(n) | Read::Scan { max: n, .. } => assert!(*n <= 4096),
                        Read::Nothing | Read::Line { .. } | Read::Arrived { .. } => panic!("tokenizer framing"),
                    }
                    pending = true;
                }
                Down::Send(_) | Down::Finish => panic!("the decoder only reads"),
            }
            wire.take(down);
        }
        match decoder.waiting() {
            Waiting::Bytes => assert!(pending),
            Waiting::Decode | Waiting::Close | Waiting::Nothing => assert!(!pending),
        }
    }
    panic!("seed {seed} failed to settle under {settings:?}");
}
/// Roomy defaults for the nine documents; generated worlds draw smaller ones.
pub const DEFAULTS: Limits = SHIPPED;

/// Decode a declaration after the stream world has checked its JSON against the reference.
///
/// # Errors
/// Returns the stream or JSON fault, or the declaration's first shape fault.
pub fn declare(
    document: &[u8],
    settings: &Settings,
    seed: u64,
) -> Result<hearth_protocol_config::declaration::Declaration, Error> {
    let run = check(document, settings, seed);
    match run.outcome.expect("declaration world runs Decode to completion") {
        Outcome::Decoded => {
            hearth_protocol_config::declaration::declaration(run.value.expect("decoded value"), &settings.limits)
        }
        Outcome::Failed(error) => Err(error),
    }
}
