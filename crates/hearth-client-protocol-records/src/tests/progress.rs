use crate::{
    errors::{Class, Error, Kind, StoreError, StoreFault, StoreLimit},
    form::{self, Form},
    limits,
    progress::{Strays, Waiting},
    records::Record,
};
const WAITING: &[(Form, &[u8])] = &[
    (Form::Text, b"waiting for the store's lock: /store\n"),
    (Form::JsonLines, b"{\"record\":\"version\",\"client\":1}\n{\"record\":\"waiting\",\"store\":\"/store\"}\n"),
];
#[test]
fn the_waiting_notice_prints_as_text_and_as_json_lines() {
    let record = Record::Waiting(Waiting { store: b"/store".as_slice().into() });
    for &(form, expected) in WAITING {
        assert_eq!(form::print(form, &record, None, &limits::SHIPPED).as_deref(), Ok(expected), "{form:?}");
    }
}
const STRAYS: &[(Form, &[u8])] = &[
    (Form::Text, b"strays in the store /store: 2, the first cache/a\n"),
    (Form::JsonLines, b"{\"record\":\"version\",\"client\":1}\n{\"record\":\"strays\",\"store\":\"/store\",\"count\":2,\"first\":\"cache/a\"}\n"),
];
#[test]
fn the_strays_notice_gives_the_count_and_the_first() {
    let record =
        Record::Strays(Strays { store: b"/store".as_slice().into(), count: 2, first: b"cache/a".as_slice().into() });
    for &(form, expected) in STRAYS {
        assert_eq!(form::print(form, &record, None, &limits::SHIPPED).as_deref(), Ok(expected), "{form:?}");
    }
}
const ERRORS: &[(Form, StoreFault, &[u8])] = &[
    (Form::Text, StoreFault::Full { limit: StoreLimit::CacheFiles, most: 2 }, b"the store /store is full: 2 cache files\n"),
    (Form::Text, StoreFault::Full { limit: StoreLimit::Resolutions, most: 3 }, b"the store /store is full: 3 resolutions\n"),
    (Form::JsonLines, StoreFault::Full { limit: StoreLimit::CacheFiles, most: 2 }, b"{\"record\":\"version\",\"client\":1}\n{\"record\":\"error\",\"kind\":\"store\",\"store\":\"/store\",\"limit\":\"cache_files\",\"most\":2,\"class\":\"operational\"}\n"),
];
#[test]
fn the_store_error_names_the_store_and_the_limit() {
    for &(form, fault, expected) in ERRORS {
        let record = Record::Error(Error {
            kind: Kind::Store(StoreError { store: b"/store".as_slice().into(), fault }),
            class: Class::Operational,
            at: None,
            also: None,
            target: None,
            task: None,
        });
        assert_eq!(form::print(form, &record, None, &limits::SHIPPED).as_deref(), Ok(expected), "{form:?} {fault:?}");
    }
}
