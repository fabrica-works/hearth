use super::{ABC, CACHE_FILE, CACHE_OWNER, Layer, REQUEST};
use crate::{Asked, builder, render};
use skein_io::file::{Event, Request};
use skein_lib::bytes;

pub fn keep_layer(bytes: &[u8]) -> Layer {
    let mut layer = Layer::new();
    layer.down(render::vocabulary::Request::Keep {
        request: REQUEST,
        path: bytes::copy_of(b"tasks/motd/tree/etc/issue.j2"),
        sha256: ABC,
        bytes: bytes::copy_of(bytes),
    });
    layer
}
pub fn synced_keep(bytes: &[u8]) -> (Layer, Box<[u8]>) {
    let mut layer = keep_layer(bytes);
    let Some(Asked::File(Request::Create { name, .. })) = layer.asked() else { panic!("create") };
    layer.file_up(Event::Opened { owner: CACHE_OWNER, file: CACHE_FILE, len: 0 });
    assert_eq!(
        layer.asked(),
        Some(Asked::File(Request::WriteAt {
            owner: CACHE_OWNER,
            file: CACHE_FILE,
            offset: 0,
            bytes: bytes::copy_of(bytes)
        }))
    );
    layer.file_up(Event::Written { owner: CACHE_OWNER });
    assert_eq!(layer.asked(), Some(Asked::File(Request::Sync { owner: CACHE_OWNER, file: CACHE_FILE })));
    layer.file_up(Event::Synced { owner: CACHE_OWNER });
    assert_eq!(layer.asked(), Some(Asked::File(Request::Close { owner: CACHE_OWNER, file: CACHE_FILE })));
    layer.file_up(Event::Closed { owner: CACHE_OWNER });
    (layer, name)
}
pub fn produced_keep() -> (Layer, Box<[u8]>) {
    let (mut layer, name) = synced_keep(b"abc");
    assert_eq!(layer.asked(), None);
    let Some(builder::vocabulary::Event::Produced { .. }) = layer.told() else { panic!("produced") };
    (layer, name)
}
pub fn cat_file_failure() -> render::vocabulary::Failure {
    render::vocabulary::Failure::Tool(render::errors::Tool {
        command: render::errors::Command::CatFile,
        ended: render::errors::Ended::Exit(1),
        tail: Box::new([]),
    })
}
