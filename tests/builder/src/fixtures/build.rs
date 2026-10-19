//! A build's scenario, and a config layer's record planted beneath the world's store.
use super::{BUILDER, GIT_DIR, Scenario, Shared, hex_digest};
use hearth::Call;
use hearth_fake_machine::linux;
use hearth_lib::encode::Digest;
use hearth_protocol_tree::layer::{Counts, Inputs, Layer, LayerKind, Made, Subvolume, TreeFile};

/// THIN's one target, built at HEAD.
#[must_use]
pub fn building() -> Scenario<'static> {
    Scenario {
        call: Call::Build {
            repository: GIT_DIR.into(),
            revision: b"HEAD".as_slice().into(),
            target: b"x1-prod".as_slice().into(),
        },
        ..super::thin()
    }
}
/// A config layer's record whose key recomputes from its parent and part (store.md, 6.2).
#[must_use]
pub fn config_layer(key: Digest, software: Digest, part: Digest) -> Layer {
    Layer {
        changes: Digest([4; 32]),
        costs: Box::new([]),
        counts: Counts { bytes: 0, deletes: 0, entries: 1, files: 0, hashed: 0, puts: 0 },
        inputs: Inputs::Config { part },
        key,
        kind: LayerKind::Config,
        made: Made { at: skein_lib::Wall::EPOCH, packages: None },
        parent: Some(software),
        placed: None,
        scan: Some(Digest([5; 32])),
        subvolume: Subvolume { parent_uuid: Some([6; 16]), uuid: [7; 16] },
        tools: Box::new([]),
        tree: Digest([8; 32]),
        tree_file: TreeFile { sha256: Digest([9; 32]), size: 0 },
    }
}
/// A config layer's record over this software key and part, its key H("config", software, part).
#[must_use]
pub fn config_over(software: Digest, part: Digest) -> Layer {
    let mut hash = hearth_lib::hash::H::new(b"config", 2);
    hash.values().digest(&software);
    hash.values().digest(&part);
    config_layer(hash.finish(), software, part)
}
/// Lays `layers/<key>.record` beneath the world's store, and `layers/` first.
pub fn plant(shared: &Shared, record: &Layer) {
    let bytes = hearth_protocol_tree::layer::write(record, &hearth_protocol_tree::limits::SHIPPED).expect("a record");
    let path = format!("/srv/store/layers/{}.record", hex_digest(record.key.0));
    let mut fake = shared.borrow_mut();
    let linux = fake.behind.machine_mut().linux_mut();
    if linux.facts(b"/srv/store/layers").is_none() {
        linux.lay(&[linux::Item::directory(b"/srv/store/layers", 0o700).owner(BUILDER)]);
    }
    linux.lay(&[linux::Item::file(path.as_bytes(), 0o644, &bytes).owner(BUILDER)]);
}
