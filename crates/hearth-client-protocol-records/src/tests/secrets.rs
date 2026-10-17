use super::fixtures;
use crate::{
    form::{self, Form},
    limits,
};
#[test]
fn secrets_print_as_choices_7_shows() {
    let text = form::print(Form::Text, &fixtures::secret_spec(), None, &limits::SHIPPED).expect("spec");
    assert_eq!(
        text.as_ref(),
        concat!(
            "target       x1-prod   base x1   environment prod\n",
            "placement    metal efi 7th-gen\n",
            "spec         9a0c5e1f2b3d\n",
            "secrets      part 000000000000\n",
            "  secret     tls/key /etc/key 0640 root:ops\n",
            "  password   free users/free.passwd\n",
            "  identity   ssh ssh-host-key ed25519 /etc/ssh/key /etc/ssh/key.pub 0600 root:root\n"
        )
        .as_bytes()
    );
}
#[test]
fn json_nests_the_record_under_secrets() {
    let json = form::print(Form::JsonLines, &fixtures::secret_spec(), None, &limits::SHIPPED).expect("spec");
    let start = skein_lib::bytes::find(&json, br#"{"record":"part","target":"x1-prod","name":"secrets""#)
        .expect("secrets part");
    let part = json.get(start..).expect("part suffix");
    assert_eq!(
        part,
        concat!(r#"{"record":"part","target":"x1-prod","name":"secrets","digest":"0000000000000000000000000000000000000000000000000000000000000000","secrets":{"files":{"tls/key":{"group":"ops","mode":"0640","owner":"root","path":"/etc/key"}},"identity":{"ssh":{"algorithm":"ed25519","group":"root","kind":"ssh-host-key","mode":"0600","owner":"root","path":"/etc/ssh/key","public_path":"/etc/ssh/key.pub"}},"passwords":{"free":"users/free.passwd"}}}"#, "\n").as_bytes()
    );
}
