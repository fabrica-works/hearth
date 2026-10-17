use super::fixtures::{self, CLIENT, Layer};
use crate::{
    Asked, Form,
    builder::{
        render::vocabulary::Outcome,
        vocabulary::{Answer, Request},
    },
    limits,
};
use skein_io as io;
use skein_lib::{
    ReplyTo, Token,
    stream::{Down, Fault, Up},
};

#[test]
fn the_terminal_record_follows_the_last_spec_then_the_stream_finishes() {
    let mut layer = Layer::with_form(Form::JsonLines, limits::SHIPPED);
    layer.bind_output();
    layer.client(Request::Answer(Answer::Spec(fixtures::spec())));
    drop(layer.printed());
    layer.client(Request::Reply { to: ReplyTo::new(Token::new(0)), outcome: Outcome::Done, diff: None });
    let requests = layer.printed();
    let [Down::Demand { read: _, room: _ }, Down::Send(bytes), Down::Finish] = requests.as_ref() else {
        panic!("terminal is printed before Finish");
    };
    assert_eq!(
        &**bytes, b"{\"record\":\"outcome\",\"outcome\":\"done\",\"left_out\":0}\n",
        "terminal follows the spec without a second version"
    );
}
#[test]
fn a_broken_output_exits_75_after_the_terminal_reply_and_close() {
    let mut layer = Layer::new();
    layer.bind_output();
    layer.client(Request::Answer(Answer::Spec(fixtures::spec())));
    let Some(Asked::Io(io::Request::Stream { stream: CLIENT, down: Down::Demand { .. } })) = layer.asked() else {
        panic!("pending output room");
    };
    layer.up(io::Event::Stream { owner: CLIENT, up: Up::Failed(Fault::Other) });
    drop(layer.printed());
    assert_eq!(layer.code(), None, "output failure still waits for the call's reply");
    layer.client(Request::Reply { to: ReplyTo::new(Token::new(0)), outcome: Outcome::Usage, diff: None });
    assert_eq!(layer.code(), Some(75), "a broken standard output has code 75");
}
const TERMINALS: &[(Outcome, u8)] =
    &[(Outcome::Done, 0), (Outcome::Failed, 2), (Outcome::Usage, 64), (Outcome::TryAgain, 75)];
#[test]
fn a_terminal_record_selects_the_exit_code_after_its_stream_closes() {
    for (outcome, expected) in TERMINALS {
        let mut layer = Layer::new();
        layer.bind_output();
        layer.client(Request::Reply { to: ReplyTo::new(Token::new(0)), outcome: *outcome, diff: None });
        assert_eq!(layer.code(), None, "the stream still owns resources");
        drop(layer.printed());
        assert_eq!(layer.code(), Some(*expected), "terminal {outcome:?}");
    }
}
#[test]
fn a_refused_spec_is_left_out_and_the_call_ends_failed() {
    let mut limits = limits::SHIPPED;
    limits.printer.record = 1;
    let mut layer = Layer::with_form(Form::Text, limits);
    layer.bind_output();
    layer.client(Request::Answer(Answer::Spec(fixtures::spec())));
    assert_eq!(layer.printed().as_ref(), &[], "a refused spec prints no bytes");
    layer.client(Request::Reply { to: ReplyTo::new(Token::new(0)), outcome: Outcome::Done, diff: None });
    drop(layer.printed());
    assert_eq!(layer.code(), Some(2), "refusing the spec makes the call Failed");
}
