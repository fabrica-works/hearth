use crate::{
    build::{Failed, Kind, Layer, State, Step, Why},
    form::{self, Form},
    limits,
    outcome::{self, Outcome},
    records::{Record, Terminal, Type},
};
const KEY: [u8; 32] = [0xab; 32];
const RECORDS: &[(Record, &[u8], &[u8])] = &[
    (
        Record::Layer(Layer { kind: Kind::Packages, key: KEY, state: State::ToBuild }),
        b"packages     abababababab   to build\n",
        b"{\"record\":\"layer\",\"layer\":\"packages\",\"key\":\"abababababababababababababababababababababababababababababababab\",\"state\":\"to_build\"}\n",
    ),
    (
        Record::Layer(Layer { kind: Kind::Config, key: KEY, state: State::Found }),
        b"config       abababababab   found\n",
        b"{\"record\":\"layer\",\"layer\":\"config\",\"key\":\"abababababababababababababababababababababababababababababababab\",\"state\":\"found\"}\n",
    ),
    (
        Record::Failed(Failed { layer: Kind::Packages, step: Step::Fetch, why: Why::Unavailable }),
        b"failed       fetch packages   unavailable\n",
        b"{\"record\":\"failed\",\"layer\":\"packages\",\"step\":\"fetch\",\"why\":\"unavailable\"}\n",
    ),
];
#[test]
fn a_builds_records_print_as_text_and_as_json_lines() {
    for (record, text, json) in RECORDS {
        assert_eq!(form::print(Form::Text, record, Some(Type::Layer), &limits::SHIPPED).as_deref(), Ok(*text));
        assert_eq!(form::print(Form::JsonLines, record, Some(Type::Layer), &limits::SHIPPED).as_deref(), Ok(*json));
    }
}
#[test]
fn an_unavailable_build_ends_with_69() {
    let record = Record::Terminal(Terminal { outcome: Outcome::Unavailable, diff: None, left_out: 0 });
    assert_eq!(form::print(Form::Text, &record, Some(Type::Failed), &limits::SHIPPED).as_deref(), Ok(b"".as_slice()));
    assert_eq!(
        form::print(Form::JsonLines, &record, Some(Type::Failed), &limits::SHIPPED).as_deref(),
        Ok(b"{\"record\":\"outcome\",\"outcome\":\"unavailable\",\"left_out\":0}\n".as_slice())
    );
    assert_eq!(outcome::code(Outcome::Unavailable), 69, "client/README.md, 3.3: unavailable");
}
