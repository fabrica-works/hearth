use super::fixtures::{self, Layer, STORE_OWNER};
use crate::{
    Asked,
    builder::{
        store::vocabulary::{Directory, Event, Request, Shape, Why},
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

const CACHE_NAMES: &[(&[u8], skein_io::kernel::Kind, Shape)] = &[
    (
        b"0101010101010101010101010101010101010101010101010101010101010101",
        skein_io::kernel::Kind::File,
        Shape::File(crate::render::vocabulary::Digest([1; 32])),
    ),
    (
        b"0101010101010101010101010101010101010101010101010101010101010101.record",
        skein_io::kernel::Kind::File,
        Shape::Record(crate::render::vocabulary::Digest([1; 32])),
    ),
    (
        b"0101010101010101010101010101010101010101010101010101010101010101.tmp",
        skein_io::kernel::Kind::File,
        Shape::Temporary,
    ),
    (
        b"0101010101010101010101010101010101010101010101010101010101010101.record.skein-0123456789abcdef",
        skein_io::kernel::Kind::File,
        Shape::Temporary,
    ),
    (b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", skein_io::kernel::Kind::File, Shape::Stray),
    (b"010101010101010101010101010101010101010101010101010101010101010", skein_io::kernel::Kind::File, Shape::Stray),
    (
        b"0101010101010101010101010101010101010101010101010101010101010101.tmp",
        skein_io::kernel::Kind::Directory,
        Shape::Stray,
    ),
    (b"notes.tmp", skein_io::kernel::Kind::File, Shape::Stray),
];
#[test]
fn a_page_of_the_cache_comes_up_as_typed_names() {
    for &(name, kind, shape) in CACHE_NAMES {
        let mut layer = Layer::new();
        fixtures::store_cursor(&mut layer, Directory::Cache, 1);
        drop(layer.asked());
        layer.file_up(file::Event::Listed {
            owner: STORE_OWNER,
            entries: Box::new([file::Entry { name: bytes::copy_of(name), kind }]),
        });
        assert_eq!(
            layer.told(),
            Some(BuilderEvent::Store(Event::Page {
                entries: Box::new([crate::builder::store::vocabulary::Entry { name: bytes::copy_of(name), shape }]),
                last: false
            })),
            "{name:?}"
        );
    }
}
const TOP_NAMES: &[(&[u8], Shape)] = &[
    (b"store.record", Shape::Fixed),
    (b"cache", Shape::Fixed),
    (b"resolutions", Shape::Fixed),
    (b"staging", Shape::Fixed),
    (b"layers", Shape::Fixed),
    (b"targets", Shape::Fixed),
    (b"state", Shape::Fixed),
    (b"artifacts", Shape::Fixed),
    (b"sources", Shape::Fixed),
    (b"caches", Shape::Fixed),
    (b"repo", Shape::Fixed),
    (b"credentials", Shape::Fixed),
    (b"store.record.skein-0123456789abcdef", Shape::Temporary),
    (b"notes.tmp", Shape::Stray),
    (b"store.record.skein-0123456789abcdeF", Shape::Stray),
];
#[test]
fn the_top_names_store_md_2_1s_fixed_names_and_the_records_temporary() {
    for &(name, shape) in TOP_NAMES {
        assert_eq!(crate::store::names::shape(Directory::Top, name, skein_io::kernel::Kind::File), shape, "{name:?}");
    }
}
#[test]
fn a_cursor_closes_after_its_last_page() {
    let mut layer = Layer::new();
    fixtures::store_cursor(&mut layer, Directory::Cache, 1024);
    assert_eq!(
        layer.asked(),
        Some(Asked::File(file::Request::ListPage { owner: Token::new(20), directory: Token::new(88), max: 1024 }))
    );
    layer.file_up(file::Event::Listed { owner: STORE_OWNER, entries: Box::new([]) });
    assert_eq!(layer.told(), None);
    assert_eq!(layer.asked(), Some(Asked::File(file::Request::Close { owner: Token::new(20), file: Token::new(88) })));
    layer.file_up(file::Event::Closed { owner: STORE_OWNER });
    assert_eq!(layer.told(), Some(BuilderEvent::Store(Event::Page { entries: Box::new([]), last: true })));
}
#[test]
fn a_failed_page_closes_its_cursor_without_listing_again() {
    let mut layer = Layer::new();
    fixtures::store_cursor(&mut layer, Directory::Cache, 1024);
    drop(layer.asked());
    layer.file_up(file::Event::Failed { owner: STORE_OWNER, error: Error::NotFound, committed: false, residue: None });
    assert_eq!(layer.told(), None);
    assert_eq!(layer.asked(), Some(Asked::File(file::Request::Close { owner: Token::new(20), file: Token::new(88) })));
    layer.file_up(file::Event::Closed { owner: STORE_OWNER });
    assert_eq!(layer.told(), Some(BuilderEvent::Store(Event::Failed)));
    assert_eq!(layer.asked(), None);
}
