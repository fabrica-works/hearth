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
    let mut builder = Builder::new(&fixtures::LIMITS);
    let mut out = Queue::with_capacity(1);
    step(
        &mut builder,
        &fixtures::env(),
        Event::Call { reply_to: ReplyTo::new(Token::new(1)), call: fixtures::call() },
        &mut out,
    );
    assert_eq!(
        out.pop(),
        Some(Request::Render(crate::render::vocabulary::Request::List {
            request: Token::new(1),
            repository: bytes::copy_of(b"/repo"),
            revision: bytes::copy_of(b"HEAD")
        })),
        "the root forwards render's request"
    );
}
#[test]
fn renders_spec_reaches_the_client() {
    let mut builder = Builder::new(&fixtures::LIMITS);
    let mut out = Queue::with_capacity(1);
    builder.output.push(Output::Answer(RenderAnswer::Spec(fixtures::spec())));
    builder.forward(&mut out);
    let Some(Request::Answer(Answer::Spec(spec))) = out.pop() else {
        panic!("client spec");
    };
    assert_eq!(&*spec.meta.target, b"x1-prod", "render's target");
    assert_eq!(spec.digest, hearth_lib::encode::Digest([2; 32]), "render's digest");
}
#[test]
#[should_panic(expected = "one builder call at a time")]
fn a_second_call_while_one_is_held_panics() {
    let mut builder = Builder::new(&fixtures::LIMITS);
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
