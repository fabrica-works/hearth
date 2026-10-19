use super::fixtures::{GROUP_IDS, GROUP_MASTER, PASSWD_IDS, PASSWD_MASTER};
use crate::{
    File, Ids, Line, Password, Refused, What,
    limits::{self, Limits},
    read, worst_case,
};
use skein_lib::{Writer, bytes};
const LIMITS: Limits = limits::SHIPPED;
fn refused(file: File, bytes: &[u8]) -> Refused {
    read(file, bytes, &LIMITS).unwrap_err()
}
fn at(number: u32, what: What) -> Refused {
    Refused::Line { number, what }
}
fn join(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).unwrap();
    }
    let mut out = Writer::new(length);
    for part in parts {
        out.put(part).unwrap();
    }
    out.finish()
}
/// A name of `length` bytes.
fn long(length: usize) -> Box<[u8]> {
    let mut name = bytes::zeroed(length);
    name.fill(b'n');
    name
}
/// Each password field in a line of `file` reads as expected.
fn passwords(file: File, cases: &[(&[u8], Password)]) {
    let (before, after): (&[u8], &[u8]) = match file {
        File::Passwd => (b"a:", b":1:1:g:/h:/s\n"),
        File::Group => (b"a:", b":1:\n"),
        File::Shadow => (b"a:", b":19000:0:99999:7:::\n"),
        File::Gshadow => (b"a:", b"::\n"),
    };
    for (password, expected) in cases {
        let lines = read(file, &join(&[before, password, after]), &LIMITS).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].password, *expected, "{file:?} {password:?}");
    }
}
/// `count` lines of a group file, each of a name of its own.
fn many(count: u32) -> Box<[u8]> {
    let mut out = Writer::new(usize::try_from(count).unwrap().checked_mul(9).unwrap());
    for number in 0..count {
        let mut name = [0_u8; 3];
        for (digit, shift) in name.iter_mut().zip([12_u32, 6, 0]) {
            let value = u8::try_from(number.wrapping_shr(shift) & 63).unwrap();
            *digit = b'A'.checked_add(value).unwrap();
        }
        out.put(&name).unwrap();
        out.put(b":x:1:\n").unwrap();
    }
    out.finish()
}
#[test]
fn the_passwd_master_reads_to_its_entries() {
    let lines = read(File::Passwd, PASSWD_MASTER, &LIMITS).unwrap();
    assert_eq!(lines.len(), PASSWD_IDS.len());
    for (line, (name, uid, gid)) in lines.iter().zip(PASSWD_IDS) {
        assert_eq!(*line.name, **name);
        assert_eq!(line.ids, Ids::User { uid: *uid, gid: *gid });
        assert_eq!(line.password, Password::Locked);
    }
}
#[test]
fn the_group_master_reads_to_its_entries() {
    let lines = read(File::Group, GROUP_MASTER, &LIMITS).unwrap();
    assert_eq!(lines.len(), GROUP_IDS.len());
    for (line, (name, gid)) in lines.iter().zip(GROUP_IDS) {
        assert_eq!(*line.name, **name);
        assert_eq!(line.ids, Ids::Group { gid: *gid });
        assert_eq!(line.password, Password::Locked);
    }
}
#[test]
fn an_empty_file_has_no_lines() {
    for file in [File::Passwd, File::Group, File::Shadow, File::Gshadow] {
        assert!(read(file, b"", &LIMITS).unwrap().is_empty());
    }
}
#[test]
fn a_last_line_may_lack_its_newline() {
    let lines = read(File::Group, b"root:x:0:\nstaff:x:50:root", &LIMITS).unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(&*lines[1].name, b"staff");
    assert_eq!(lines[1].ids, Ids::Group { gid: 50 });
}
#[test]
fn passwd_and_group_passwords_are_locked_as_x_or_star() {
    for file in [File::Passwd, File::Group] {
        passwords(
            file,
            &[
                (b"x", Password::Locked),
                (b"*", Password::Locked),
                (b"", Password::Set),
                (b"!", Password::Set),
                (b"xx", Password::Set),
                (b"X", Password::Set),
                (b"$y$j9T$salt$hash", Password::Set),
            ],
        );
    }
}
#[test]
fn shadow_and_gshadow_passwords_are_locked_as_star_or_bangs() {
    for file in [File::Shadow, File::Gshadow] {
        passwords(
            file,
            &[
                (b"*", Password::Locked),
                (b"!", Password::Locked),
                (b"!!", Password::Locked),
                (b"!*", Password::Locked),
                (b"", Password::Set),
                (b"x", Password::Set),
                (b"!!!", Password::Set),
                (b"*!", Password::Set),
                (b"$y$j9T$salt$hash", Password::Set),
                (b"!$6$salt$hash", Password::Set),
            ],
        );
    }
}
#[test]
fn shadow_and_gshadow_carry_no_ids() {
    let shadow = read(File::Shadow, b"root:*:19000:0:99999:7:::\n", &LIMITS).unwrap();
    assert_eq!(shadow[0], Line { name: Box::from(&b"root"[..]), ids: Ids::Shadow, password: Password::Locked });
    let gshadow = read(File::Gshadow, b"root:!::\n", &LIMITS).unwrap();
    assert_eq!(gshadow[0].ids, Ids::Shadow);
}
#[test]
fn fields_past_those_named_are_not_interpreted() {
    let lines = read(File::Passwd, b"a:x:1:2:\xff\r:/h home:\n", &LIMITS).unwrap();
    assert_eq!(lines[0].ids, Ids::User { uid: 1, gid: 2 });
    let lines = read(File::Shadow, b"a:!:x:-1:y:z:::\n", &LIMITS).unwrap();
    assert_eq!(lines[0].password, Password::Locked);
}
#[test]
fn a_carriage_return_is_a_byte_like_any_other() {
    // The CR ends the members' field, which is not interpreted.
    assert_eq!(read(File::Group, b"a:x:1:\r\n", &LIMITS).unwrap().len(), 1);
    assert_eq!(refused(File::Group, b"a:x:1\r:\n"), at(1, What::Id));
    assert_eq!(refused(File::Group, b"\r\n"), at(1, What::Fields));
    assert_eq!(&*read(File::Group, b"a\r:x:1:\n", &LIMITS).unwrap()[0].name, b"a\r");
}
#[test]
fn the_wrong_count_of_fields_is_refused() {
    assert_eq!(refused(File::Passwd, b"a:x:1:1:g:/h\n"), at(1, What::Fields));
    assert_eq!(refused(File::Passwd, b"a:x:1:1:g:/h:/s:\n"), at(1, What::Fields));
    assert_eq!(refused(File::Group, b"root:x:0:\nstaff:x:50\n"), at(2, What::Fields));
    assert_eq!(refused(File::Group, b"a:x:1::\n"), at(1, What::Fields));
    assert_eq!(refused(File::Shadow, b"a:*:1:0:9:7:\n"), at(1, What::Fields));
    assert_eq!(refused(File::Gshadow, b"a:*:\n"), at(1, What::Fields));
}
#[test]
fn an_empty_line_a_comment_and_a_nis_line_are_refused() {
    assert_eq!(refused(File::Group, b"root:x:0:\n\nstaff:x:50:\n"), at(2, What::Fields));
    assert_eq!(refused(File::Group, b"\n"), at(1, What::Fields));
    assert_eq!(refused(File::Group, b"# a comment\n"), at(1, What::Fields));
    assert_eq!(refused(File::Group, b"#a:x:1:\n"), at(1, What::Name));
    assert_eq!(refused(File::Passwd, b"+::::::\n"), at(1, What::Name));
    assert_eq!(refused(File::Passwd, b"+@netgroup:::::\n"), at(1, What::Fields));
    assert_eq!(refused(File::Passwd, b"-user:x:1:1:g:/h:/s\n"), at(1, What::Name));
    assert_eq!(refused(File::Shadow, b"+::::::::\n"), at(1, What::Name));
}
#[test]
fn a_name_is_not_empty_and_at_most_the_limit() {
    assert_eq!(refused(File::Group, b":x:1:\n"), at(1, What::Name));
    assert_eq!(read(File::Group, &join(&[&long(256), b":x:1:\n"]), &LIMITS).unwrap().len(), 1);
    assert_eq!(refused(File::Group, &join(&[&long(257), b":x:1:\n"])), at(1, What::Name));
    let small = Limits { name: 3, ..LIMITS };
    assert_eq!(read(File::Group, b"abc:x:1:\n", &small).unwrap().len(), 1);
    assert_eq!(read(File::Group, b"abcd:x:1:\n", &small).unwrap_err(), at(1, What::Name));
}
#[test]
fn an_id_is_decimal_digits_without_a_leading_zero() {
    for id in [&b""[..], b"+1", b"-1", b"1a", b" 1", b"01", b"00", b"0x1", b"1.0", "\u{661}".as_bytes()] {
        assert_eq!(refused(File::Group, &join(&[b"a:x:", id, b":\n"])), at(1, What::Id), "{id:?}");
    }
    assert_eq!(read(File::Group, b"a:x:0:\n", &LIMITS).unwrap()[0].ids, Ids::Group { gid: 0 });
    assert_eq!(refused(File::Passwd, b"a:x:1:01:g:/h:/s\n"), at(1, What::Id));
}
#[test]
fn an_id_is_at_most_65535() {
    assert_eq!(read(File::Group, b"a:x:65535:\n", &LIMITS).unwrap()[0].ids, Ids::Group { gid: 65535 });
    assert_eq!(refused(File::Group, b"a:x:65536:\n"), at(1, What::Range));
    assert_eq!(refused(File::Group, b"a:x:99999:\n"), at(1, What::Range));
    assert_eq!(refused(File::Group, b"a:x:4294967296:\n"), at(1, What::Range));
    assert_eq!(refused(File::Group, b"a:x:99999999999999999999999999:\n"), at(1, What::Range));
    assert_eq!(refused(File::Passwd, b"a:x:65535:65536:g:/h:/s\n"), at(1, What::Range));
    assert_eq!(refused(File::Passwd, b"a:x:65536:x:g:/h:/s\n"), at(1, What::Range));
}
#[test]
fn ids_may_repeat_and_names_may_not() {
    assert_eq!(read(File::Group, b"a:x:5:\nb:x:5:\n", &LIMITS).unwrap().len(), 2);
    assert_eq!(refused(File::Group, b"a:x:5:\nb:x:6:\na:x:7:\n"), at(3, What::Duplicate));
    assert_eq!(refused(File::Shadow, b"a:*:::::::\na:!:::::::\n"), at(2, What::Duplicate));
    // A name is compared whole, by its bytes.
    assert_eq!(read(File::Group, b"a:x:5:\nA:x:5:\nab:x:5:\n", &LIMITS).unwrap().len(), 3);
}
#[test]
fn the_first_line_that_fails_is_the_one_reported() {
    assert_eq!(refused(File::Group, b"a:x:1:\na:x:99999:\nb\n"), at(2, What::Range));
    assert_eq!(refused(File::Group, b"a:x:1:\na:x:1:\nb\n"), at(2, What::Duplicate));
}
#[test]
fn a_file_past_the_limit_is_refused_whole() {
    let small = Limits { file: 9, ..LIMITS };
    assert_eq!(read(File::Group, b"a:x:1:\nb:x", &small), Err(Refused::File));
    assert_eq!(read(File::Group, b"a:x:1:\nb:", &small), Err(at(2, What::Fields)));
    assert_eq!(read(File::Group, b"a:x:1:\n", &small).unwrap().len(), 1);
}
#[test]
fn more_lines_than_the_limit_are_refused_before_any_is_read() {
    let small = Limits { lines: 2, ..LIMITS };
    assert_eq!(read(File::Group, b"a:x:1:\nb:x:1:\n", &small).unwrap().len(), 2);
    assert_eq!(read(File::Group, b"a:x:1:\nb:x:1:\nc:x:1:\n", &small), Err(Refused::Lines));
    assert_eq!(read(File::Group, b"a:x:1:\nb:x:1:\nc", &small), Err(Refused::Lines));
    // Even where the first line would fail.
    assert_eq!(read(File::Group, b"\n\n\n", &small), Err(Refused::Lines));
}
#[test]
fn the_shipped_limits_are_the_designs() {
    assert_eq!(limits::SHIPPED, Limits { file: 1_048_576, lines: 4_096, name: 256 });
    let bytes = many(4_097);
    assert_eq!(read(File::Group, &bytes, &LIMITS), Err(Refused::Lines));
    assert_eq!(read(File::Group, &bytes[..bytes.len() - 1], &LIMITS), Err(Refused::Lines));
    assert_eq!(read(File::Group, &many(4_096), &LIMITS).unwrap().len(), 4_096);
}
#[test]
fn the_worst_case_covers_the_list_and_the_names() {
    let worst = worst_case(&LIMITS).unwrap();
    assert!(worst >= 4_096 * u64::try_from(size_of::<Line>()).unwrap() + 1_048_576 - 1);
    assert_eq!(worst_case(&Limits { file: 0, lines: 0, name: 0 }), Some(0));
    assert!(worst_case(&Limits { file: u32::MAX, lines: u32::MAX, name: u32::MAX }).is_some());
}
#[test]
fn an_error_names_a_line_and_what_was_wrong_not_the_content() {
    // A refusal holds a line number and a reason, and nothing else: no content can be in one.
    assert_eq!(refused(File::Shadow, b"root:$y$j9T$secret:::\n"), at(1, What::Fields));
}
