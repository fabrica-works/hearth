use crate::{
    limits::{self, Limits},
    listing::{Entry, Error, Event, Mode},
    tests::fixtures,
};
const RECORD: &[u8] = b"100644 blob ce013625030ba8dba906f756967f9e9ca394464a       6\tx\0";
#[test]
fn a_listing_record_of_each_mode_decodes_to_its_class() {
    let stdout = b"100644 blob ce013625030ba8dba906f756967f9e9ca394464a       6\ta b.json\0\
100755 blob 1a2485251c33a70432394c93fb89330ef214bfc9      10\tbin/x\0\
120000 blob d0f12b94eeb3c5301ed4468f5c26e689c113ae10      11\tlink\0\
160000 commit ce013625030ba8dba906f756967f9e9ca394464a       -\tsub\0";
    let Event::Listed(entries) = fixtures::listed(stdout, &limits::SHIPPED) else {
        panic!("git's own listing is listed")
    };
    let expected = [(Mode::File, 6_u64), (Mode::Executable, 10), (Mode::Link, 11), (Mode::Submodule, 0)];
    assert_eq!(entries.len(), expected.len(), "a record each");
    for (entry, (mode, size)) in entries.iter().zip(expected) {
        assert_eq!((entry.mode, entry.size), (mode, size), "its class, its size");
    }
}
#[test]
fn a_size_not_padded_as_git_pads_it_is_malformed() {
    const SIZES: [&[u8]; 7] =
        [b"6", b"     6", b"       6", b"     06", b"      -", b"       ", b"18446744073709551616"];
    for size in SIZES {
        let mut writer = skein_lib::Writer::new(56_usize.checked_add(size.len()).expect("small size"));
        writer.put(b"100644 blob ce013625030ba8dba906f756967f9e9ca394464a ").expect("header");
        writer.put(size).expect("size");
        writer.put(b"\tx\0").expect("path");
        assert_eq!(
            fixtures::listed(&writer.finish(), &limits::SHIPPED),
            Event::Failed(Error::Malformed { entry: 0 }),
            "bad padding {size:?}"
        );
    }
}
#[test]
fn an_uppercase_or_short_object_id_is_malformed() {
    const CASES: [&[u8]; 2] =
        [b"100644 blob Ce013625030ba8dba906f756967f9e9ca394464a       6\tx\0", b"100644 blob ce0136       6\tx\0"];
    for input in CASES {
        assert_eq!(
            fixtures::listed(input, &limits::SHIPPED),
            Event::Failed(Error::Malformed { entry: 0 }),
            "strict id"
        );
    }
}
#[test]
fn the_entry_past_the_limit_fails_naming_its_path() {
    let limits = Limits { entries: 0, ..limits::SHIPPED };
    assert_eq!(
        fixtures::listed(RECORD, &limits),
        Event::Failed(Error::TooMany { path: Box::from(b"x".as_slice()) }),
        "excess entry"
    );
}
#[test]
fn the_entry_whose_path_passes_the_total_fails_naming_its_path() {
    let limits = Limits { paths_total: 64, ..limits::SHIPPED };
    let stdout = b"100644 blob ce013625030ba8dba906f756967f9e9ca394464a       6\taaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\0\
100644 blob ce013625030ba8dba906f756967f9e9ca394464a       6\tbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\0\
100644 blob ce013625030ba8dba906f756967f9e9ca394464a       6\tccccc\0";
    assert_eq!(
        fixtures::listed(stdout, &limits),
        Event::Failed(Error::PathsTooLarge { path: Box::from(b"ccccc".as_slice()) }),
        "third path crosses total"
    );
}
#[test]
fn a_path_past_the_limit_fails_with_its_prefix() {
    const CASES: [&[u8]; 2] = [b"100644 blob ce013625030ba8dba906f756967f9e9ca394464a       6\tabcd\0", b"100644 blob ce013625030ba8dba906f756967f9e9ca394464a       6\tabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz\0"];
    let limits = Limits { path: 3, ..limits::SHIPPED };

    for input in CASES {
        assert_eq!(
            fixtures::listed(input, &limits),
            Event::Failed(Error::PathTooLong { prefix: Box::from(b"abc".as_slice()) }),
            "path prefix"
        );
    }
}
#[test]
fn an_end_within_a_record_is_truncated() {
    for length in 1..RECORD.len() {
        assert_eq!(
            fixtures::listed(&RECORD[..length], &limits::SHIPPED),
            Event::Failed(Error::Truncated),
            "cut at {length}"
        );
    }
    assert_eq!(fixtures::listed(b"", &limits::SHIPPED), Event::Listed(Box::<[Entry]>::from([])), "boundary end");
}
