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
        "store.md, 3.2: H(\"empty\")"
    );
    assert_eq!(
        keys.packages,
        digest(b"22d5156ffefadb1f0df4eb2c4d405b2c8937215e044486b06b1725e3506b238d"),
        "store.md, 3.2: H(\"packages\", digest 00…00, digest 01…01)"
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
const STARTS: &[(Option<Layer>, &[Layer], Option<Step>)] = &[
    (None, &[Layer::Packages, Layer::Software, Layer::Config], Some(Step::Fetch(Layer::Packages))),
    (Some(Layer::Packages), &[Layer::Software, Layer::Config], Some(Step::Fetch(Layer::Software))),
    (Some(Layer::Software), &[Layer::Config], Some(Step::Users)),
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
            (Output::Request(Request::Run { step }), Some(first)) => assert_eq!(step, first, "{published:?}"),
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
#[test]
fn an_unavailable_step_fails_the_build_with_its_plan() {
    let mut build = Build::new();
    let (_, output) = drive(&mut build, None);
    assert_eq!(output, Output::Request(Request::Run { step: Step::Fetch(Layer::Packages) }));
    let mut out = Queue::with_capacity(crate::MAX_OUT);
    step(&mut build, &env(), Event::Unavailable { step: Step::Fetch(Layer::Packages) }, &mut out);
    let Some(Output::Answer(Answer::Failed { plan, step, why })) = out.pop() else { panic!("the terminal") };
    assert_eq!((step, why), (Step::Fetch(Layer::Packages), Why::Unavailable));
    assert_eq!(plan, crate::plan::plan(keys(CLOSURE, PACKAGES, SOFTWARE, CONFIG), None));
    assert!(build.is_empty(), "the build ends with its terminal");
}
#[test]
fn the_plan_fits_its_worst_case() {
    assert!(crate::limits::worst_case(&SHIPPED).is_some(), "the shipped plan's bound is finite");
}
