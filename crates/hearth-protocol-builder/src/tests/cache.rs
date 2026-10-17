use super::fixtures::{self, CACHE_FILE as FILE, CACHE_OWNER as OWNER};
use skein_io::{
    file::{Event, Request},
    kernel::Fd,
};
use skein_lib::bytes;
#[test]
fn a_cached_blob_is_written_to_a_temporary_synced_then_renamed() {
    let mut cache = fixtures::cache();
    let Some(Request::Create { owner, root, name, mode }) = cache.take_request() else { panic!("temporary create") };
    assert_eq!((owner, root, mode, name.len()), (OWNER, Fd::new(4), 0o666, 68));
    assert_eq!(name.get(64..), Some(b".tmp".as_slice()));
    cache.up(Event::Opened { owner: OWNER, file: FILE, len: 0 });
    cache.piece(bytes::copy_of(b"abc"));
    assert_eq!(
        cache.take_request(),
        Some(Request::WriteAt { owner: OWNER, file: FILE, offset: 0, bytes: bytes::copy_of(b"abc") })
    );
    cache.up(Event::Written { owner: OWNER });
    cache.finish();
    assert_eq!(cache.take_request(), Some(Request::Sync { owner: OWNER, file: FILE }));
    cache.up(Event::Synced { owner: OWNER });
    assert_eq!(cache.take_request(), Some(Request::Close { owner: OWNER, file: FILE }));
    cache.up(Event::Closed { owner: OWNER });
    assert_eq!(cache.take_request(), None, "the store decides publication");
    assert!(cache.done());
}
#[test]
fn a_cache_file_already_present_keeps_it_and_removes_the_temporary() {
    let mut cache = fixtures::synced_cache();
    assert_eq!(cache.take_request(), None, "the index decides presence without probing the cache");
    assert!(cache.done());
}
#[test]
fn a_failed_git_waits_for_the_inflight_cache_open_and_close() {
    let (mut layer, owner) = fixtures::opening_cache();
    layer.up(skein_io::Event::Exited { owner, exit: skein_io::kernel::Exit::Code(1) });
    assert_eq!(layer.asked(), None, "the open still owns its completion");
    layer.file_up(Event::Opened { owner: OWNER, file: FILE, len: 0 });
    assert_eq!(layer.asked(), Some(crate::Asked::File(Request::Close { owner: OWNER, file: FILE })));
    layer.file_up(Event::Closed { owner: OWNER });
    assert_eq!(
        layer.run(b"", skein_io::kernel::Exit::Code(1)),
        crate::builder::vocabulary::Event::Render(crate::render::vocabulary::Event::Failed {
            request: fixtures::REQUEST,
            class: crate::render::vocabulary::Class::Operational,
            at: None,
            failure: fixtures::cat_file_failure()
        })
    );
}

#[test]
fn a_kept_file_is_written_hashed_synced_then_renamed() {
    let (mut layer, name) = fixtures::produced_keep();
    layer.client(crate::builder::vocabulary::Request::Store(crate::builder::store::vocabulary::Request::Keep {
        arrival: skein_lib::Token::new(10),
        record: fixtures::cache_record(crate::builder::store::vocabulary::Origin::Rendered {
            path: bytes::copy_of(b"tasks/motd/tree/etc/issue.j2"),
        }),
    }));
    assert_eq!(
        layer.asked(),
        Some(crate::Asked::File(Request::Rename {
            owner: fixtures::STORE_OWNER,
            from_root: fixtures::roots().cache.fd,
            to_root: fixtures::roots().cache.fd,
            from: name,
            to: bytes::copy_of(b"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        }))
    );
    layer.file_up(Event::Renamed { owner: fixtures::STORE_OWNER });
    assert_eq!(
        layer.asked(),
        Some(crate::Asked::File(Request::SyncDirectory {
            owner: fixtures::STORE_OWNER,
            root: fixtures::roots().cache.fd
        }))
    );
    layer.file_up(Event::Synced { owner: fixtures::STORE_OWNER });
    let Some(crate::Asked::File(Request::Store { path, .. })) = layer.asked() else {
        panic!("record after durable rename")
    };
    assert_eq!(path.as_ref(), b"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad.record");
    layer.file_up(Event::Stored { owner: fixtures::STORE_OWNER, digest: skein_io::digest::digest(b"record") });
    assert_eq!(
        layer.told(),
        Some(crate::builder::vocabulary::Event::Store(crate::builder::store::vocabulary::Event::Done))
    );
}
#[test]
fn a_kept_file_already_present_removes_its_temporary() {
    let (mut layer, name) = fixtures::produced_keep();
    layer.client(crate::builder::vocabulary::Request::Store(crate::builder::store::vocabulary::Request::Discard {
        arrival: skein_lib::Token::new(10),
    }));
    assert_eq!(
        layer.asked(),
        Some(crate::Asked::File(Request::Remove {
            owner: fixtures::STORE_OWNER,
            root: fixtures::roots().cache.fd,
            name,
            directory: false
        }))
    );
    layer.file_up(Event::Removed { owner: fixtures::STORE_OWNER });
    assert_eq!(
        layer.told(),
        Some(crate::builder::vocabulary::Event::Store(crate::builder::store::vocabulary::Event::Done))
    );
}
#[test]
fn a_keep_whose_bytes_are_not_its_sha256_fails_and_leaves_no_temporary() {
    let (mut layer, name) = fixtures::synced_keep(b"wrong");
    assert_eq!(
        layer.asked(),
        Some(crate::Asked::File(Request::Remove {
            owner: OWNER,
            root: fixtures::roots().cache.fd,
            name,
            directory: false
        }))
    );
    layer.file_up(Event::Removed { owner: OWNER });
    assert_eq!(layer.asked(), None);
    assert_eq!(
        layer.told(),
        Some(crate::builder::vocabulary::Event::Render(crate::render::vocabulary::Event::Failed {
            request: fixtures::REQUEST,
            class: crate::render::vocabulary::Class::Operational,
            at: None,
            failure: crate::render::vocabulary::Failure::Store
        }))
    );
}
