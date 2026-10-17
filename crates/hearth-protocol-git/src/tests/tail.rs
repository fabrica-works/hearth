use crate::{
    limits::{self, Limits},
    tail::Tail,
};
const SMALL: Limits = Limits { tail_lines: 2, tail_line: 8, ..limits::SHIPPED };
#[test]
fn the_tail_keeps_the_last_lines_each_cut_to_its_limit() {
    let mut tail = Tail::new(&SMALL);
    tail.admit(b"error: one\nerr");
    tail.admit(b"or: two\nfatal: not a tree object\n");
    assert_eq!(
        tail.lines().as_ref(),
        &[skein_lib::bytes::copy_of(b"error: t"), skein_lib::bytes::copy_of(b"fatal: n")]
    );
}
#[test]
fn a_last_line_without_a_line_feed_is_kept() {
    let mut tail = Tail::new(&SMALL);
    tail.admit(b"one\ntwo\nthree");
    assert_eq!(tail.lines().as_ref(), &[skein_lib::bytes::copy_of(b"two"), skein_lib::bytes::copy_of(b"three")]);
}
