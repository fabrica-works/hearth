use super::fixtures;
use crate::{revision::Oid, vocabulary::As};
use skein_lib::bytes;

struct Documents {
    refused: bool,
    expected: &'static [(&'static [u8], As)],
}
const DOCUMENTS: &[Documents] = &[
    Documents { refused: false, expected: &[(fixtures::reuse::CHANGED_DOCUMENT, As::Document)] },
    Documents {
        refused: true,
        expected: &[
            (fixtures::reuse::CHANGED_DOCUMENT, As::Document),
            (fixtures::reuse::REFUSED_DOCUMENT, As::Document),
        ],
    },
];
#[test]
fn a_diffs_second_side_reads_only_the_blobs_that_changed() {
    for row in DOCUMENTS {
        let read = fixtures::reuse::documents(row.refused);
        assert_eq!(fixtures::reuse::blobs(&read).as_ref(), row.expected, "refused {}", row.refused);
    }
}
#[test]
fn a_diffs_second_side_past_64_mib_fails_at_the_path_a_render_alone_names() {
    let read = fixtures::reuse::budget();
    assert_eq!(fixtures::kept(&read.side).as_ref(), &[fixtures::reuse::budget_error()]);
}
#[test]
fn uncounted_kept_blobs_go_before_a_needed_read_is_kept() {
    let read = fixtures::reuse::budget();
    assert_eq!(
        read.blobs.as_ref(),
        &[
            (Oid(bytes::copy_of(fixtures::reuse::NEW_DOCUMENT)), As::Document),
            (Oid(bytes::copy_of(b"envs/d.json")), As::Document)
        ]
    );
}
