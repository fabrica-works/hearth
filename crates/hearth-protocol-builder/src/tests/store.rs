use super::fixtures::{self, Layer, STORE_OWNER};
use crate::{
    Asked,
    builder::{
        store::vocabulary::{Event, Request, Why},
        vocabulary::{Event as BuilderEvent, Request as BuilderRequest},
    },
    tree,
};
use skein_io::{file, kernel::Error};
use skein_lib::{ReplyTo, Token, bytes};
#[test]
fn a_lock_asks_io_through_the_stores_root_token() {
    let mut layer = Layer::new();
    layer.client(BuilderRequest::Store(Request::Lock));
    assert_eq!(layer.asked(), Some(Asked::File(file::Request::Lock { owner: Token::new(20), root: Token::new(31) })));
}
#[test]
fn a_lock_timed_out_is_told_busy() {
    let mut layer = Layer::new();
    layer.client(BuilderRequest::Store(Request::Lock));
    drop(layer.asked());
    layer.file_up(file::Event::Failed { owner: STORE_OWNER, error: Error::TimedOut, committed: false, residue: None });
    assert_eq!(layer.told(), Some(BuilderEvent::Store(Event::Busy)));
}
#[test]
fn an_absent_store_record_is_none_and_a_written_one_is_framed_as_kind_1() {
    let mut layer = Layer::new();
    layer.client(BuilderRequest::Store(Request::LoadRecord));
    assert_eq!(
        layer.asked(),
        Some(Asked::File(file::Request::Load {
            owner: Token::new(20),
            root: Token::new(31),
            path: bytes::copy_of(b"store.record"),
            max: 4120,
            no_follow: true
        }))
    );
    layer.file_up(file::Event::Failed { owner: STORE_OWNER, error: Error::NotFound, committed: false, residue: None });
    assert_eq!(layer.told(), Some(BuilderEvent::Store(Event::Loaded { record: None })));
    layer.client(BuilderRequest::Store(Request::WriteRecord { record: fixtures::store_record() }));
    let Some(Asked::File(file::Request::Store { bytes, expected, no_follow, .. })) = layer.asked() else {
        panic!("store write")
    };
    assert_eq!(expected, file::Expect::Any);
    assert!(no_follow);
    assert_eq!(
        tree::store::read(&bytes, &tree::limits::SHIPPED),
        Ok(tree::store::Store { layout: 1, made: skein_lib::Wall::from_nanos(77), range: None, uid: 1000 })
    );
}
const STARTUP: &[(Why, &[u8])] = &[
    (Why::Lock, b"hearth: cannot open the store: lock failed\n"),
    (Why::Record, b"hearth: cannot open the store: store.record failed\n"),
    (
        Why::Owner { recorded: 1001, builder: 1000 },
        b"hearth: cannot open the store: store.record is uid 1001's, not uid 1000's\n",
    ),
];
#[test]
fn an_unopened_store_writes_its_startup_record() {
    for &(why, expected) in STARTUP {
        let mut layer = Layer::new();
        layer.bind_output();
        layer.client(BuilderRequest::Unopened { to: ReplyTo::new(Token::new(0)), why });
        assert_eq!(layer.startup().as_deref(), Some(expected), "{why:?}");
        let printed = layer.printed();
        assert_eq!(&*printed, &[skein_lib::stream::Down::Finish], "no standard output");
    }
}

#[test]
fn an_unopened_store_exits_75() {
    let mut layer = Layer::new();
    layer.bind_output();
    layer.client(BuilderRequest::Unopened { to: ReplyTo::new(Token::new(0)), why: Why::Lock });
    drop(layer.printed());
    assert_eq!(layer.code(), Some(75));
}
