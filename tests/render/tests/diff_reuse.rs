use hearth_domain_render::{diff::Side, limits::SHIPPED, vocabulary::As};
use hearth_render_world::{fixtures, parent};

struct Documents {
    refused: bool,
    expected: &'static [(&'static [u8], As)],
}
const DOCUMENTS: &[Documents] = &[
    Documents { refused: false, expected: &[(fixtures::reuse::NEW_OID, As::Document)] },
    Documents {
        refused: true,
        expected: &[(fixtures::reuse::REFUSED_OID, As::Document), (fixtures::reuse::NEW_OID, As::Document)],
    },
];
#[test]
fn a_diffs_second_side_reads_only_the_blobs_that_changed() {
    for row in DOCUMENTS {
        let from = fixtures::reuse::repository(row.refused);
        let to = fixtures::reuse::changed_document(&from);
        let run = parent::diff(&from, &to, fixtures::reuse::SEED, SHIPPED);
        assert_eq!(fixtures::reuse::second_reads(&run), row.expected, "refused {}", row.refused);
    }
}
#[test]
fn a_diffs_second_side_errs_as_a_render_of_its_revision_alone() {
    let from = fixtures::reuse::repository(true);
    let to = fixtures::reuse::moved_and_changed(&from);
    let diff = parent::diff(&from, &to, fixtures::reuse::SEED, SHIPPED);
    let alone = parent::run(&to, &[], fixtures::reuse::SEED);
    let errors = fixtures::reuse::errors(&diff, Some(Side::To));
    assert_eq!(errors, fixtures::reuse::errors(&alone, None));
    assert_eq!(
        fixtures::reuse::error_paths(&errors),
        [b"envs/renamed.json".as_slice(), b"tasks/motd/tree/etc/renamed.j2".as_slice()]
    );
}

const INTERPRETATIONS: &[(fixtures::reuse::Interpretation, (&[u8], As))] = &[
    (fixtures::reuse::Interpretation::StaticTemplate, (fixtures::reuse::STATIC_OID, As::Template)),
    (fixtures::reuse::Interpretation::TemplateStatic, (fixtures::reuse::TEMPLATE_OID, As::Cached)),
    (fixtures::reuse::Interpretation::LinkStatic, (fixtures::reuse::LINK_OID, As::Cached)),
];
#[test]
fn a_blob_read_as_something_else_is_read_again() {
    let from = fixtures::reuse::repository(false);
    for (interpretation, expected) in INTERPRETATIONS {
        let to = fixtures::reuse::reinterpreted(&from, *interpretation);
        let run = parent::diff(&from, &to, fixtures::reuse::SEED, SHIPPED);
        assert_eq!(fixtures::only(&fixtures::reuse::second_reads(&run)), expected, "{interpretation:?}");
    }
}
