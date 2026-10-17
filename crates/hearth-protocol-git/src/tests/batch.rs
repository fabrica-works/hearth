use crate::{
    batch::{self, Error, Event, Request},
    limits::{self, Limits},
    oid::Oid,
    tests::fixtures,
};
fn oid() -> Oid {
    Oid::parse(b"ce013625030ba8dba906f756967f9e9ca394464a").expect("literal id")
}
#[test]
fn a_blob_s_header_body_and_line_feed_read_whole() {
    let limits = Limits { chunk: 4, ..limits::SHIPPED };
    let stdout = b"ce013625030ba8dba906f756967f9e9ca394464a blob 6\nhello\n\n";
    let events = fixtures::batched(stdout, &[Request::Read { oid: oid(), size: 6, max: 6 }], &limits);
    assert_eq!(
        &*events,
        &[
            Event::Piece(Box::from(b"hell".as_slice())),
            Event::Piece(Box::from(b"o\n".as_slice())),
            Event::Whole { oid: oid(), size: 6 }
        ],
        "pieces then whole"
    );
}
#[test]
fn a_header_whose_size_is_not_the_listing_s_fails_first() {
    let stdout = b"ce013625030ba8dba906f756967f9e9ca394464a blob 6\n";
    let events = fixtures::batched(stdout, &[Request::Read { oid: oid(), size: 7, max: 10 }], &limits::SHIPPED);
    assert_eq!(&*events, &[Event::Failed(Error::Size { oid: oid(), listed: 7, header: 6 })], "before body");
}
#[test]
fn a_blob_past_its_kind_s_maximum_fails_before_its_body() {
    let stdout = b"ce013625030ba8dba906f756967f9e9ca394464a blob 6\n";
    let events = fixtures::batched(stdout, &[Request::Read { oid: oid(), size: 6, max: 5 }], &limits::SHIPPED);
    assert_eq!(&*events, &[Event::Failed(Error::TooLarge { oid: oid(), size: 6, max: 5 })], "before body");
}
#[test]
fn a_missing_object_answers_missing_and_the_batch_reads_on() {
    let stdout =
        b"ce013625030ba8dba906f756967f9e9ca394464a missing\nce013625030ba8dba906f756967f9e9ca394464a blob 0\n\n";
    let requests = [Request::Read { oid: oid(), size: 9, max: 9 }, Request::Read { oid: oid(), size: 0, max: 0 }];
    assert_eq!(
        &*fixtures::batched(stdout, &requests, &limits::SHIPPED),
        &[Event::Missing { oid: oid() }, Event::Whole { oid: oid(), size: 0 }],
        "missing does not end batch"
    );
}
#[test]
fn a_body_not_ended_by_a_line_feed_is_malformed() {
    let stdout = b"ce013625030ba8dba906f756967f9e9ca394464a blob 0\nx";
    assert_eq!(
        &*fixtures::batched(stdout, &[Request::Read { oid: oid(), size: 0, max: 0 }], &limits::SHIPPED),
        &[Event::Failed(Error::Malformed)],
        "required separator"
    );
}
#[test]
fn the_request_line_is_the_id_and_a_line_feed() {
    const CASES: [(&[u8], &[u8]); 2] = [
        (b"ce013625030ba8dba906f756967f9e9ca394464a", b"ce013625030ba8dba906f756967f9e9ca394464a\n"),
        (
            b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\n",
        ),
    ];
    for (hex, expected) in CASES {
        assert_eq!(&*batch::request_line(Oid::parse(hex).expect("literal")), expected, "one request");
    }
}
#[test]
fn a_header_for_another_object_or_kind_is_refused() {
    const CASES: [&[u8]; 2] =
        [b"ce013625030ba8dba906f756967f9e9ca394464a tree 0\n", b"0000000000000000000000000000000000000000 blob 0\n"];
    for stdout in CASES {
        let events = fixtures::batched(stdout, &[Request::Read { oid: oid(), size: 0, max: 0 }], &limits::SHIPPED);
        let other = Oid::parse(&stdout[..40]).expect("literal id");
        assert_eq!(&*events, &[Event::Failed(Error::Other { oid: other })], "object and blob kind");
    }
}
#[test]
fn an_end_within_header_body_or_separator_is_truncated() {
    let stdout = b"ce013625030ba8dba906f756967f9e9ca394464a blob 6\nhello\n\n";
    for at in 0..stdout.len() {
        let events =
            fixtures::batched(&stdout[..at], &[Request::Read { oid: oid(), size: 6, max: 6 }], &limits::SHIPPED);
        assert_eq!(events.last(), Some(&Event::Failed(Error::Truncated)), "end at {at}");
    }
}

#[test]
fn a_fault_while_idle_or_between_pieces_waits_for_the_next_request() {
    use skein_lib::{
        Env, Queue, Time, Wall, bytes,
        stream::{Fault, Up},
    };
    for paused in [false, true] {
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits::SHIPPED };
        let mut machine = batch::Batch::START;
        let mut above = Queue::with_capacity(1);
        let mut below = Queue::with_capacity(1);
        let read = Request::Read { oid: oid(), size: 1, max: 1 };
        if paused {
            batch::down(&mut machine, &env, read, &mut above, &mut below);
            drop(below.pop());
            for input in [b"c".as_slice(), b"e013625030ba8dba906f756967f9e9ca394464a blob 1\n", b"x"] {
                batch::up(&mut machine, &env, Up::Bytes(bytes::copy_of(input)), &mut above, &mut below);
                drop(below.pop());
            }
            assert_eq!(above.pop(), Some(Event::Piece(Box::from(b"x".as_slice()))), "piece answered Read");
        }
        batch::up(&mut machine, &env, Up::Failed(Fault::Reset), &mut above, &mut below);
        assert_eq!(above.pop(), None, "no request awaits an answer");
        batch::down(&mut machine, &env, if paused { Request::Next } else { read }, &mut above, &mut below);
        assert_eq!(above.pop(), Some(Event::Failed(Error::Stream(Fault::Reset))), "next demand gets held fault");
        assert_eq!(below.pop(), None, "no read after broken stream");
    }
}
