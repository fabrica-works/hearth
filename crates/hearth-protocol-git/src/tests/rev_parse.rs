const BAD: [&[u8]; 6] = [
    b"",
    b"ce013625030ba8dba906f756967f9e9ca394464a",
    b"CE013625030ba8dba906f756967f9e9ca394464a\n",
    b"ce01362\n",
    b"ce013625030ba8dba906f756967f9e9ca394464a\r\n",
    b"ce013625030ba8dba906f756967f9e9ca394464a\n\n",
];
use crate::{
    invocation::Exit,
    oid::Oid,
    rev_parse::{self, Error, Resolved},
};
#[test]
fn rev_parse_s_one_line_decodes_to_the_commit_it_names() {
    const CASES: [(&[u8], &[u8]); 2] = [
        (b"ce013625030ba8dba906f756967f9e9ca394464a\n", b"ce013625030ba8dba906f756967f9e9ca394464a"),
        (
            b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\n",
            b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        ),
    ];
    for (stdout, hex) in CASES {
        let oid = Oid::parse(hex).expect("valid literal");
        assert_eq!(rev_parse::decode(stdout, Exit::Code(0)), Ok(Resolved::Commit(oid)), "one line");
        assert_eq!(oid.hex(), hex, "id retains its width");
    }
    for stdout in BAD {
        assert_eq!(rev_parse::decode(stdout, Exit::Code(0)), Err(Error::Malformed), "malformed {stdout:?}");
    }
}
#[test]
fn rev_parse_exiting_1_is_unresolved_whatever_it_printed() {
    const CASES: [&[u8]; 3] =
        [b"", b"garbage", b"ce013625030ba8dba906f756967f9e9ca394464a\n^ce013625030ba8dba906f756967f9e9ca394464a\n"];
    for stdout in CASES {
        assert_eq!(rev_parse::decode(stdout, Exit::Code(1)), Ok(Resolved::Unresolved), "exit 1 overrides stdout");
    }
}
#[test]
fn an_exit_past_1_or_a_signal_is_a_tool_failure() {
    for exit in [Exit::Code(2), Exit::Code(128), Exit::Code(255), Exit::Signal(9), Exit::Signal(15)] {
        assert_eq!(rev_parse::decode(b"", exit), Err(Error::Exit(exit)), "tool exit retained");
    }
}
