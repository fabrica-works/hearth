use super::fixtures::data;
use crate::{
    form::{self, Form},
    limits,
};
#[test]
fn data_prints_its_names_and_paths_as_9_2_shows() {
    assert_eq!(form::print(Form::Text, &data::spec(), None, &limits::SHIPPED).expect("data").as_ref(), concat!("target       x1-prod   base x1   environment prod\nplacement    metal efi 7th-gen\nspec         9a0c5e1f2b3d\ndata         part 000000000000\n", "  iwd        /var/lib/iwd\n  journal    /var/log/journal\n").as_bytes());
}
#[test]
fn data_json_keeps_names_classes_and_paths() {
    let json = form::print(Form::JsonLines, &data::spec(), None, &limits::SHIPPED).expect("data");
    let start = skein_lib::bytes::find(&json, br#"{"record":"part","target":"x1-prod","name":"data""#).expect("part");
    assert_eq!(json.get(start..).expect("part"), concat!(r#"{"record":"part","target":"x1-prod","name":"data","digest":"0000000000000000000000000000000000000000000000000000000000000000","data":{"entries":{"iwd":{"class":{"kind":"core"},"path":"/var/lib/iwd"},"journal":{"class":{"kind":"core"},"path":"/var/log/journal"}}}}"#, "\n").as_bytes());
}
#[test]
fn a_data_diff_prints_the_new_class_and_path() {
    assert_eq!(
        form::print(Form::Text, &data::change(), None, &limits::SHIPPED).expect("change").as_ref(),
        concat!(
            "x1-prod                                             new slot only\n",
            "  data     ~ iwd                                    data /var/lib/iwd\n"
        )
        .as_bytes()
    );
}
#[test]
fn a_data_diff_json_keeps_both_values() {
    assert_eq!(form::print(Form::JsonLines, &data::change(), Some(crate::records::Type::Change), &limits::SHIPPED).expect("change").as_ref(), concat!(r#"{"record":"change","target":"x1-prod","kind":"changed","changed":{"parts":[{"kind":"data","data":{"entries":[{"after":{"class":{"kind":"data"},"path":"/var/lib/iwd"},"before":{"class":{"kind":"core"},"path":"/var/lib/iwd"},"key":"iwd","kind":"changed"}]}}],"rebuild":{"kind":"slot_only"}}}"#, "\n").as_bytes());
}
