use super::fixtures;
use crate::{
    form::{self, Form},
    limits,
};
#[test]
fn software_prints_as_choices_7_shows() {
    let text = form::print(Form::Text, &fixtures::software_spec(), None, &limits::SHIPPED).expect("spec");
    assert_eq!(text.as_ref(), concat!(
        "target       x1-prod   base x1   environment prod\nplacement    metal efi 7th-gen\nspec         9a0c5e1f2b3d\nsoftware     part 000000000000\n",
        "  download   /opt/a file 0755 root:staff 010101010101\n",
        "  download   /opt/b tar 1 root:staff 010101010101\n",
        "  scripts    motd 300 010101010101\n",
        "  fetch      010101010101 https://example.org/a\n"
    ).as_bytes());
}
#[test]
fn json_nests_the_record_under_software() {
    let json = form::print(Form::JsonLines, &fixtures::software_spec(), None, &limits::SHIPPED).expect("spec");
    let start = skein_lib::bytes::find(&json, br#"{"record":"part","target":"x1-prod","name":"software""#)
        .expect("software part");
    assert_eq!(json.get(start..).expect("suffix"), concat!(r#"{"record":"part","target":"x1-prod","name":"software","digest":"0000000000000000000000000000000000000000000000000000000000000000","software":{"downloads":{"/opt/a":{"group":"staff","mode":"0755","owner":"root","sha256":"0101010101010101010101010101010101010101010101010101010101010101","unpack":"file"},"/opt/b":{"group":"staff","owner":"root","sha256":"0101010101010101010101010101010101010101010101010101010101010101","strip":1,"unpack":"tar"}},"fetch":{"0101010101010101010101010101010101010101010101010101010101010101":"https://example.org/a"},"scripts":[{"content_digest":"0101010101010101010101010101010101010101010101010101010101010101","order":300,"task":"motd"}]}}"#, "\n").as_bytes());
}
