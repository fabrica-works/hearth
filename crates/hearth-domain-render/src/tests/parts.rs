use super::fixtures;
use crate::parts::meta;
use hearth_lib::encode::Encoder;
#[test]
fn the_meta_record_encodes_as_store_md_3_1_writes_it() {
    let meta = fixtures::meta();
    let expected = fixtures::hex(
        "07000000000000000404000000000000000462617365040000000000000002783104000000000000000b656e7669726f6e6d656e7404000000000000000470726f64040000000000000009706c6163656d656e740700000000000000030400000000000000086669726d7761726504000000000000000365666904000000000000000868617264776172650400000000000000073774682d67656e0400000000000000046b696e640400000000000000056d6574616c04000000000000000674617267657404000000000000000778312d70726f64",
    );
    let mut measure = Encoder::measuring();
    meta::encode(&meta, &mut measure);
    let mut encoder = Encoder::writing(measure.measured());
    meta::encode(&meta, &mut encoder);
    assert_eq!(encoder.written(), expected, "store.md, 3.1: meta record");
}
#[test]
fn the_meta_digest_is_h_of_part_meta_over_its_record() {
    assert_eq!(
        meta::digest(&fixtures::meta()),
        fixtures::digest("706777e0f8563a881bb75631bb73470298bc6d9f2f638d1dc526310adeb9b73d"),
        "render.md, 4.4"
    );
}

const MODES: &[(crate::revision::Mode, u16)] =
    &[(crate::revision::Mode::Executable, 0o755), (crate::revision::Mode::File, 0o644)];
#[test]
fn an_executable_blob_is_0755_and_any_other_file_0644() {
    for &(mode, expected) in MODES {
        let entry = crate::parts::config::file(b"etc/motd", mode, hearth_lib::encode::Digest([0; 32]));
        let crate::parts::config::Node::File { mode, .. } = entry.node;
        assert_eq!(mode, expected, "git mode");
    }
}
#[test]
fn the_config_digest_is_h_of_part_config_over_its_tree() {
    assert_eq!(
        crate::parts::config::digest(&fixtures::config()),
        fixtures::digest("e2475ba7ef36cd9f9b7b38622b1f67c97f173151988357e2ba4e7226661c1fd2"),
        "store.md 3.1's tree encoding"
    );
}

#[test]
fn a_task_removed_by_null_in_the_base_env_gives_no_files() {
    let (mut call, target) = fixtures::removed_task();
    call.targets = Box::new([target]);
    call.commit = Some(crate::revision::Oid(skein_lib::bytes::copy_of(b"commit")));
    let crate::vocabulary::Answer::Spec(spec) = call.answer() else { panic!("removed task still renders its target") };
    let [part] = spec.parts.as_ref() else { panic!("config part") };
    let crate::parts::PartRecord::Config(config) = &part.record;
    assert_eq!(config.tree.as_ref(), [], "null removes the task's files");
}
