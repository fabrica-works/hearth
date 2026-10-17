use super::fixtures;
use crate::{
    form::{self, Form},
    limits,
    records::Record,
};
#[test]
fn boot_parameters_and_mutability_print_as_render_md_9_2_shows() {
    assert!(
        skein_lib::bytes::find(
            &fixtures::packages_text(fixtures::boot_spec()),
            b"boot         part bc6cab417252\n  params     quiet audit=0\n  mutable    true\n"
        )
        .is_some()
    );
}
#[test]
fn json_gives_the_boot_record_in_key_order() {
    let json =
        form::print(Form::JsonLines, &Record::Spec(fixtures::boot_spec()), None, &limits::SHIPPED).expect("boot json");
    let start =
        skein_lib::bytes::find(&json, br#"{"record":"part","target":"x1-prod","name":"boot""#).expect("boot part");
    assert_eq!(json.get(start..).expect("part suffix"), concat!(r#"{"record":"part","target":"x1-prod","name":"boot","digest":"bc6cab4172520000000000000000000000000000000000000000000000000000","boot":{"mutable":true,"params":["quiet","audit=0"]}}"#, "\n").as_bytes());
}
