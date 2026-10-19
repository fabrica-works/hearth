use super::fixtures::{Layer, STORE_OWNER};
use crate::{
    Asked,
    builder::{
        build::{self, Step},
        store::vocabulary::{self as store, Directory, Event, Request, Shape},
        vocabulary::{Event as BuilderEvent, Request as BuilderRequest},
    },
    tree,
};
use hearth_lib::encode::Digest;
use skein_io::{
    file,
    kernel::{Error, Kind},
};
use skein_lib::{Token, Wall};
#[test]
fn every_step_is_answered_unavailable() {
    for step in [Step::Fetch(build::Layer::Packages), Step::Users] {
        let mut layer = Layer::new();
        layer.client(BuilderRequest::Run { step });
        assert_eq!(layer.asked(), None, "no io: no step runs yet");
        assert_eq!(layer.told(), Some(BuilderEvent::Build(build::vocabulary::Event::Unavailable { step })));
    }
}
const RECORD: &[u8] = b"0101010101010101010101010101010101010101010101010101010101010101.record";
const LAYER_NAMES: &[(&[u8], Kind, Shape)] = &[
    (RECORD, Kind::File, Shape::Record(Digest([1; 32]))),
    (b"0101010101010101010101010101010101010101010101010101010101010101", Kind::Directory, Shape::Fixed),
    (b"0101010101010101010101010101010101010101010101010101010101010101.tree", Kind::File, Shape::Fixed),
    (b"0101010101010101010101010101010101010101010101010101010101010101.staging", Kind::Directory, Shape::Fixed),
    (b"0101010101010101010101010101010101010101010101010101010101010101.trash", Kind::Directory, Shape::Fixed),
    (b"0101010101010101010101010101010101010101010101010101010101010101.tree.tmp", Kind::File, Shape::Fixed),
    (b"0101010101010101010101010101010101010101010101010101010101010101.changes", Kind::File, Shape::Fixed),
    (
        b"0101010101010101010101010101010101010101010101010101010101010101.record.skein-0123456789abcdef",
        Kind::File,
        Shape::Fixed,
    ),
    (b"0101010101010101010101010101010101010101010101010101010101010101", Kind::File, Shape::Stray),
    (RECORD, Kind::Directory, Shape::Stray),
    (b"0101010101010101010101010101010101010101010101010101010101010101.tmp", Kind::File, Shape::Stray),
    (b"notes", Kind::File, Shape::Stray),
];
#[test]
fn the_names_store_md_2_1_lists_in_layers_are_not_strays() {
    for &(name, kind, shape) in LAYER_NAMES {
        assert_eq!(crate::store::names::shape(Directory::Layers, name, kind), shape, "{name:?} {kind:?}");
    }
}
#[test]
fn an_absent_layers_directory_lists_as_one_empty_page() {
    let mut layer = Layer::new();
    layer.client(BuilderRequest::Store(Request::List { directory: Directory::Layers, max: 1024 }));
    let Some(Asked::File(file::Request::OpenDirectory { name, .. })) = layer.asked() else { panic!("open layers/") };
    assert_eq!(&*name, b"layers");
    layer.file_up(file::Event::Failed { owner: STORE_OWNER, error: Error::NotFound, committed: false, residue: None });
    assert_eq!(
        layer.told(),
        Some(BuilderEvent::Store(Event::Page { entries: Box::new([]), last: true })),
        "domain/store.md, 2.4: a store without layers"
    );
}
fn config_record() -> tree::layer::Layer {
    use tree::layer::{Counts, Inputs, LayerKind, Made, Subvolume, TreeFile};
    tree::layer::Layer {
        changes: Digest([4; 32]),
        costs: Box::new([]),
        counts: Counts { bytes: 0, deletes: 0, entries: 1, files: 0, hashed: 0, puts: 0 },
        inputs: Inputs::Config { part: Digest([2; 32]) },
        key: Digest([1; 32]),
        kind: LayerKind::Config,
        made: Made { at: Wall::from_nanos(9), packages: None },
        parent: Some(Digest([3; 32])),
        placed: None,
        scan: Some(Digest([5; 32])),
        subvolume: Subvolume { parent_uuid: Some([6; 16]), uuid: [7; 16] },
        tools: Box::new([]),
        tree: Digest([8; 32]),
        tree_file: TreeFile { sha256: Digest([9; 32]), size: 0 },
    }
}
#[test]
fn a_layer_record_loads_from_beneath_layers_and_reaches_the_store() {
    let mut layer = Layer::new();
    layer.client(BuilderRequest::Store(Request::LoadLayer { key: Digest([1; 32]) }));
    let Some(Asked::File(file::Request::Load { root, path, .. })) = layer.asked() else { panic!("load") };
    assert_eq!(
        (root, &*path),
        (Token::new(31), b"layers/0101010101010101010101010101010101010101010101010101010101010101.record".as_slice())
    );
    let bytes = tree::layer::write(&config_record(), &tree::limits::SHIPPED).expect("a config record");
    layer.file_up(file::Event::Loaded { owner: STORE_OWNER, bytes });
    assert_eq!(
        layer.told(),
        Some(BuilderEvent::Store(Event::Layer {
            record: store::Layer {
                key: Digest([1; 32]),
                parent: Some(Digest([3; 32])),
                inputs: store::LayerInputs::Config { part: Digest([2; 32]) },
                tree: Digest([8; 32]),
                uuid: [7; 16],
                at: Wall::from_nanos(9),
            }
        }))
    );
}
