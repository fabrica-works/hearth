use crate::{
    Build, Layer, Step, keys,
    limits::SHIPPED,
    plan::How,
    step,
    vocabulary::{Answer, Event, Output, Request, Why},
};
use hearth_lib::{encode::Digest, hash::H};
use skein_lib::{Env, List, Queue, Time, Wall};
const CLOSURE: Digest = Digest([0; 32]);
const PACKAGES: Digest = Digest([1; 32]);
const SOFTWARE: Digest = Digest([2; 32]);
const CONFIG: Digest = Digest([3; 32]);
const BUILD: Event = Event::Build { closure: CLOSURE, packages: PACKAGES, software: SOFTWARE, config: CONFIG };
fn written_out(tag: &[u8], beneath: &Digest, part: &Digest) -> Digest {
    let mut hash = H::new(tag, 2);
    hash.values().digest(beneath);
    hash.values().digest(part);
    hash.finish()
}
fn digest(hex: &[u8; 64]) -> Digest {
    let mut bytes = [0_u8; 32];
    for (byte, pair) in bytes.iter_mut().zip(hex.chunks_exact(2)) {
        for digit in pair {
            let value = char::from(*digit).to_digit(16).expect("a hex digit");
            *byte = byte
                .checked_mul(16)
                .expect("two digits")
                .checked_add(u8::try_from(value).expect("a digit"))
                .expect("two digits");
        }
    }
    Digest(bytes)
}
#[test]
fn the_keys_are_store_md_3_3_s() {
    let keys = keys(CLOSURE, PACKAGES, SOFTWARE, CONFIG);
    assert_eq!(
        keys.empty,
        digest(b"8f43cea6a4bf9d3aec62c882bdafeab30cc6d71acc9b0b984b7f2fd8f70975d3"),
        "domain/store.md, 3.2: H(\"empty\")"
    );
    assert_eq!(
        keys.packages,
        digest(b"22d5156ffefadb1f0df4eb2c4d405b2c8937215e044486b06b1725e3506b238d"),
        "domain/store.md, 3.2: H(\"packages\", digest 00…00, digest 01…01)"
    );
    assert_eq!(
        keys.software,
        written_out(b"software", &keys.packages, &SOFTWARE),
        "H(\"software\", packages key, part)"
    );
    assert_eq!(keys.config, written_out(b"config", &keys.software, &CONFIG), "H(\"config\", software key, part)");
}
fn env() -> Env<crate::limits::Limits> {
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits: SHIPPED }
}
/// Drives one build, answering `Have` for the published layer and `Free` above it.
fn drive(build: &mut Build, published: Option<Layer>) -> (List<Digest>, Output) {
    let mut out = Queue::with_capacity(crate::MAX_OUT);
    let keys = keys(CLOSURE, PACKAGES, SOFTWARE, CONFIG);
    step(build, &env(), BUILD, &mut out);
    let mut asked = List::with_capacity(3);
    for _ in 0_u8..4 {
        let output = out.pop().expect("one output per event");
        let Output::Request(Request::Want { key }) = output else {
            assert!(out.is_empty(), "one output");
            return (asked, output);
        };
        asked.push(key).expect("three questions at most");
        let event = match published {
            Some(layer) if keys.of(layer) == key => Event::Have { key },
            Some(_) | None => Event::Free { key },
        };
        step(build, &env(), event, &mut out);
    }
    unreachable!("three questions at most")
}
const STARTS: &[(Option<Layer>, &[Layer], Option<Layer>)] = &[
    (None, &[Layer::Packages, Layer::Software, Layer::Config], Some(Layer::Packages)),
    (Some(Layer::Packages), &[Layer::Software, Layer::Config], Some(Layer::Software)),
    (Some(Layer::Software), &[Layer::Config], Some(Layer::Config)),
    (Some(Layer::Config), &[], None),
];
const TOP_DOWN: [Layer; 3] = [Layer::Config, Layer::Software, Layer::Packages];
#[test]
fn the_plan_builds_the_layers_above_the_highest_published_bottom_up() {
    let keys = keys(CLOSURE, PACKAGES, SOFTWARE, CONFIG);
    for &(published, layers, first) in STARTS {
        let (asked, output) = drive(&mut Build::new(), published);
        let mut expected = List::with_capacity(3);
        for layer in TOP_DOWN {
            expected.push(keys.of(layer)).expect("three layers");
            if Some(layer) == published {
                break;
            }
        }
        assert_eq!(asked.as_slice(), expected.as_slice(), "{published:?}: top down, to the first published");
        match (output, first) {
            (Output::Request(Request::Stage { key, parent }), Some(first)) => {
                assert_eq!(
                    (key, parent),
                    (keys.of(first), keys.parent(first)),
                    "{published:?}: stages the lowest layer"
                );
            }
            (Output::Answer(Answer::Built { plan }), None) => {
                assert_eq!(plan.found, Some(Layer::Config), "a published config key is the output");
                assert!(plan.build.is_empty(), "nothing to build");
            }
            (output, _) => panic!("{published:?}: {output:?}"),
        }
        let plan = crate::plan::plan(keys, published);
        assert_eq!(plan.build.len(), layers.len(), "{published:?}");
        for (planned, layer) in plan.build.iter().zip(layers) {
            assert_eq!(planned.layer, *layer, "{published:?}: bottom-up");
            assert_eq!(planned.key, keys.of(planned.layer), "{published:?}: {layer:?}'s key");
            assert_eq!(&*planned.steps, crate::plan::steps(planned.layer), "{published:?}");
        }
        assert_eq!(plan.how, How::Scratch, "4.2: no base while no packages layer is reconciled");
    }
}
fn keys_of() -> crate::Keys {
    keys(CLOSURE, PACKAGES, SOFTWARE, CONFIG)
}
/// Feeds one event and takes the one output.
fn feed(build: &mut Build, event: Event) -> Output {
    let mut out = Queue::with_capacity(crate::MAX_OUT);
    step(build, &env(), event, &mut out);
    let output = out.pop().expect("one output per event");
    assert!(out.is_empty(), "one output");
    output
}
/// A build whose store has published `published`, and the request it ends planning on.
fn planned(published: Option<Layer>) -> (Build, Output) {
    let mut build = Build::new();
    let (_, output) = drive(&mut build, published);
    (build, output)
}
/// Stages `layer` and runs its steps up to, not including, `until`; the output after them.
fn run_to(build: &mut Build, layer: Layer, until: usize, mut output: Output) -> Output {
    let keys = keys_of();
    let steps = crate::plan::steps(layer);
    assert_eq!(
        output,
        Output::Request(Request::Stage { key: keys.of(layer), parent: keys.parent(layer) }),
        "{layer:?}: stage first"
    );
    output = feed(build, Event::Staged { key: keys.of(layer) });
    for step in steps.iter().take(until) {
        assert_eq!(output, Output::Request(Request::Run { step: *step }), "{layer:?}: its steps in order");
        output = feed(build, Event::Ran { step: *step });
    }
    output
}
/// Runs every step of `layer` and seals it; the output after the seal.
fn make(build: &mut Build, layer: Layer, output: Output) -> Output {
    let keys = keys_of();
    let output = run_to(build, layer, crate::plan::steps(layer).len(), output);
    assert_eq!(output, Output::Request(Request::Seal { key: keys.of(layer) }), "{layer:?}: seal after the last step");
    feed(build, Event::Sealed { key: keys.of(layer) })
}
#[test]
fn a_plan_of_three_layers_stages_runs_and_seals_each_bottom_up_and_ends_built() {
    let (mut build, mut output) = planned(None);
    for layer in crate::LAYERS {
        output = make(&mut build, layer, output);
    }
    let Output::Answer(Answer::Built { plan }) = output else { panic!("built: {output:?}") };
    assert_eq!(plan, crate::plan::plan(keys_of(), None));
    assert!(build.is_empty(), "the build ends with its terminal");
}
#[test]
fn a_plan_above_a_found_software_layer_stages_config_alone_on_the_software_key() {
    let keys = keys_of();
    let (mut build, output) = planned(Some(Layer::Software));
    assert_eq!(output, Output::Request(Request::Stage { key: keys.config, parent: keys.software }));
    let output = make(&mut build, Layer::Config, output);
    let Output::Answer(Answer::Built { plan }) = output else { panic!("built: {output:?}") };
    assert_eq!(plan.found, Some(Layer::Software));
    assert_eq!(plan.build.len(), 1, "config alone");
}
#[test]
fn the_packages_layer_is_staged_from_the_empty_layer() {
    let keys = keys_of();
    let (_, output) = planned(None);
    assert_eq!(output, Output::Request(Request::Stage { key: keys.packages, parent: keys.empty }));
}
/// How a layer goes wrong after its stage: a step at this index, or the seal.
#[derive(Clone, Copy)]
enum Trouble {
    Failed(usize),
    Unavailable(usize),
    Seal,
}
const TROUBLES: &[(Trouble, Why)] = &[
    (Trouble::Failed(1), Why::Failed),
    (Trouble::Unavailable(0), Why::Unavailable),
    (Trouble::Unavailable(2), Why::Unavailable),
    (Trouble::Seal, Why::Seal),
];
/// Brings the software layer to its trouble; the failed step, and the output after it.
fn go_wrong(build: &mut Build, trouble: Trouble, output: Output) -> (Step, Output) {
    let keys = keys_of();
    let steps = crate::plan::steps(Layer::Software);
    let (at, ran_and_failed) = match trouble {
        Trouble::Failed(at) => (at, true),
        Trouble::Unavailable(at) => (at, false),
        Trouble::Seal => {
            let output = run_to(build, Layer::Software, steps.len(), output);
            assert_eq!(output, Output::Request(Request::Seal { key: keys.software }));
            let last = steps.last().copied().expect("a layer's steps");
            return (last, feed(build, Event::Refused { key: keys.software }));
        }
    };
    let output = run_to(build, Layer::Software, at, output);
    let step = steps[at];
    assert_eq!(output, Output::Request(Request::Run { step }), "the step in flight");
    let event = if ran_and_failed { Event::StepFailed { step } } else { Event::Unavailable { step } };
    (step, feed(build, event))
}
#[test]
fn a_failure_past_stage_discards_the_layer_and_then_fails_with_the_first_cause() {
    let keys = keys_of();
    for &(trouble, why) in TROUBLES {
        for discard_refused in [false, true] {
            let (mut build, output) = planned(None);
            let output = make(&mut build, Layer::Packages, output);
            let (failed, output) = go_wrong(&mut build, trouble, output);
            assert_eq!(output, Output::Request(Request::Discard { key: keys.software }), "{why:?}: discard");
            let answered = if discard_refused {
                Event::Refused { key: keys.software }
            } else {
                Event::Discarded { key: keys.software }
            };
            let Output::Answer(Answer::Failed { plan, step, why: cause }) = feed(&mut build, answered) else {
                panic!("{why:?}: the terminal")
            };
            assert_eq!((step, cause), (failed, why), "the first cause, discard refused: {discard_refused}");
            assert_eq!(plan, crate::plan::plan(keys, None));
            assert!(build.is_empty(), "the build ends with its terminal");
        }
    }
}
#[test]
fn a_refused_stage_fails_the_build_at_once_with_no_discard() {
    let keys = keys_of();
    let (mut build, output) = planned(Some(Layer::Packages));
    assert_eq!(output, Output::Request(Request::Stage { key: keys.software, parent: keys.packages }));
    let output = feed(&mut build, Event::Refused { key: keys.software });
    let Output::Answer(Answer::Failed { step, why, .. }) = output else { panic!("{output:?}") };
    assert_eq!((step, why), (Step::Fetch(Layer::Software), Why::Stage), "the layer's first step");
    assert!(build.is_empty(), "the build ends with its terminal");
}
#[test]
fn the_plan_fits_its_worst_case() {
    assert!(crate::limits::worst_case(&SHIPPED).is_some(), "the shipped plan's bound is finite");
}
