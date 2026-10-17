use super::fixtures;
use crate::{
    form::{self, Form},
    limits,
};
#[test]
fn live_prints_as_choices_7_shows() {
    let text = form::print(Form::Text, &fixtures::live_spec(), None, &limits::SHIPPED).expect("spec");
    assert_eq!(
        text.as_ref(),
        concat!(
            "target       x1-prod   base x1   environment prod\n",
            "placement    metal efi 7th-gen\n",
            "spec         9a0c5e1f2b3d\n",
            "live         part 000000000000\n",
            "  path       /opt/tool/ motd - none\n"
        )
        .as_bytes()
    );
}
#[test]
fn json_nests_the_record_under_live() {
    let json = form::print(Form::JsonLines, &fixtures::live_spec(), None, &limits::SHIPPED).expect("spec");
    let start =
        skein_lib::bytes::find(&json, br#"{"record":"part","target":"x1-prod","name":"live""#).expect("live part");
    assert_eq!(json.get(start..).expect("suffix"), concat!(r#"{"record":"part","target":"x1-prod","name":"live","digest":"0000000000000000000000000000000000000000000000000000000000000000","live":{"/opt/tool/":[{"action":"none","task":"motd"}]}}"#, "\n").as_bytes());
}
