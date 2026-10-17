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
