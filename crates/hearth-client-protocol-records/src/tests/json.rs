use super::fixtures;
use crate::{
    form::{self, Form, Refusal},
    limits,
    outcome::Outcome,
    records::{Record, Terminal},
};
#[test]
fn json_lines_start_with_the_version_and_give_digests_whole() {
    let printed = form::print(Form::JsonLines, &Record::Spec(fixtures::x1_prod()), None, &limits::SHIPPED);
    let expected = concat!(
        r#"{"record":"version","client":1}"#,
        "\n",
        r#"{"record":"spec","target":"x1-prod","commit":"1a2b3c4d00000000000000000000000000000000","digest":"9a0c5e1f2b3d0000000000000000000000000000000000000000000000000000"}"#,
        "\n",
        r#"{"record":"part","target":"x1-prod","name":"meta","digest":"c4d3a0b1e2f50000000000000000000000000000000000000000000000000000","meta":{"base":"x1","environment":"prod","placement":{"firmware":"efi","hardware":"7th-gen","kind":"metal"},"target":"x1-prod"}}"#,
        "\n",
        r#"{"record":"part","target":"x1-prod","name":"config","digest":"e7c2a0d95b130000000000000000000000000000000000000000000000000000","config":{"groups":{},"members":[],"tree":{"/etc/hostname":{"content_digest":"3f1c2ab07e440000000000000000000000000000000000000000000000000000","group":"root","kind":"file","mode":"0644","owner":"root"}},"users":{}}}"#,
        "\n",
    );
    assert_eq!(printed.as_deref(), Ok(expected.as_bytes()), "client/README.md, 3.2: version first, digests whole");
}
#[test]
fn the_terminal_record_gives_its_outcome_by_name() {
    const ROWS: [(Outcome, &[u8]); 5] = [
        (Outcome::Done, b"{\"record\":\"outcome\",\"outcome\":\"done\",\"left_out\":0}\n"),
        (Outcome::Negative, b"{\"record\":\"outcome\",\"outcome\":\"negative\",\"left_out\":0}\n"),
        (Outcome::Failed, b"{\"record\":\"outcome\",\"outcome\":\"failed\",\"left_out\":0}\n"),
        (Outcome::Usage, b"{\"record\":\"outcome\",\"outcome\":\"usage\",\"left_out\":0}\n"),
        (Outcome::TryAgain, b"{\"record\":\"outcome\",\"outcome\":\"try_again\",\"left_out\":0}\n"),
    ];
    for (outcome, expected) in ROWS {
        let printed = form::print(
            Form::JsonLines,
            &Record::Terminal(Terminal { outcome, diff: None, left_out: 0 }),
            Some(crate::records::Type::Spec),
            &limits::SHIPPED,
        );
        assert_eq!(printed.as_deref(), Ok(expected), "terminal outcome {outcome:?}");
    }
}
#[test]
fn a_path_that_is_not_utf8_is_refused_as_text() {
    let mut spec = fixtures::x1_prod();
    let config = fixtures::config_mut(&mut spec);
    fixtures::only_mut(&mut config.tree).path = skein_lib::bytes::copy_of(b"/\xff");
    assert_eq!(
        form::print(Form::JsonLines, &Record::Spec(spec), None, &limits::SHIPPED),
        Err(Refusal::Text),
        "JSON paths must be UTF-8"
    );
}

#[test]
fn json_gives_the_resolution_and_its_keys_whole() {
    let spec = fixtures::with_resolution(fixtures::x1_prod(), fixtures::known());
    let printed = form::print(Form::JsonLines, &Record::Spec(spec), None, &limits::SHIPPED).expect("valid spec");
    let expected = concat!(
        r#"{"record":"version","client":1}"#,
        "\n",
        r#"{"record":"spec","target":"x1-prod","commit":"1a2b3c4d00000000000000000000000000000000","digest":"9a0c5e1f2b3d0000000000000000000000000000000000000000000000000000","resolution":{"key":"0c7d9e41c0d20000000000000000000000000000000000000000000000000000","keys":{"config":"3333333333333333333333333333333333333333333333333333333333333333","packages":"1111111111111111111111111111111111111111111111111111111111111111","software":"2222222222222222222222222222222222222222222222222222222222222222"},"kind":"known"}}"#,
        "\n",
        r#"{"record":"part","target":"x1-prod","name":"meta","digest":"c4d3a0b1e2f50000000000000000000000000000000000000000000000000000","meta":{"base":"x1","environment":"prod","placement":{"firmware":"efi","hardware":"7th-gen","kind":"metal"},"target":"x1-prod"}}"#,
        "\n",
        r#"{"record":"part","target":"x1-prod","name":"config","digest":"e7c2a0d95b130000000000000000000000000000000000000000000000000000","config":{"groups":{},"members":[],"tree":{"/etc/hostname":{"content_digest":"3f1c2ab07e440000000000000000000000000000000000000000000000000000","group":"root","kind":"file","mode":"0644","owner":"root"}},"users":{}}}"#,
        "\n",
    );
    assert_eq!(&*printed, expected.as_bytes(), "resolution key, state and layer keys whole");
}
