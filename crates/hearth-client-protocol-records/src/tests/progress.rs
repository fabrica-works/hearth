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
            side: None,
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

const RESOLUTION_PROGRESS: &[(Record, &[u8], &[u8])] = &[
    (Record::Resolving(crate::progress::Resolving { key: [1; 32] }), b"resolving 010101010101\n",
     b"{\"record\":\"resolving\",\"key\":\"0101010101010101010101010101010101010101010101010101010101010101\"}\n"),
    (Record::Resolved(crate::progress::Resolved { key: [1; 32], took: skein_lib::Duration::from_nanos(7_999_999) }), b"resolved 010101010101 in 7 ms\n",
     b"{\"record\":\"resolved\",\"key\":\"0101010101010101010101010101010101010101010101010101010101010101\",\"took_ms\":7}\n"),
    (Record::Unresolved(crate::progress::Unresolved { key: [1; 32] }), b"resolution 010101010101 failed\n",
     b"{\"record\":\"unresolved\",\"key\":\"0101010101010101010101010101010101010101010101010101010101010101\"}\n"),
];
#[test]
fn resolution_progress_prints_as_text_and_json_lines() {
    for (row, (record, text, json)) in RESOLUTION_PROGRESS.iter().enumerate() {
        for (form, expected) in [(Form::Text, *text), (Form::JsonLines, *json)] {
            assert_eq!(
                form::print(form, record, Some(crate::records::Type::Waiting), &limits::SHIPPED).as_deref(),
                Ok(expected),
                "row {row}: {form:?}"
            );
        }
    }
}
