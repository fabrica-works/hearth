use crate::{records::errors as records, render::errors as render};
const COMPOSITION: &[render::Composition] = &[render::Composition::Duplicate, render::Composition::Long];
#[test]
fn each_composition_kind_translates_and_back() {
    for (row, input) in COMPOSITION.iter().enumerate() {
        let back = match crate::translate::errors::composition(*input) {
            records::Composition::Duplicate => render::Composition::Duplicate,
            records::Composition::Long => render::Composition::Long,
        };
        assert_eq!(back, *input, "row {row}: composition kind through records and back");
    }
}
#[test]
fn a_document_s_members_keep_their_names_positions_and_layers() {
    use crate::render::revision::{Layer, Member, Members, Position};
    use skein_lib::bytes::copy_of;
    let document = crate::translate::documents::document(crate::tests::fixtures::member_document());
    assert_eq!(
        document.members,
        Some(Members {
            at: Position { line: 1, column: 12 },
            each: Box::new([
                Member {
                    name: copy_of(b"1"),
                    at: Position { line: 1, column: 13 },
                    layer: Some(Layer { placement: None, tasks: Box::new([]) })
                },
                Member { name: copy_of(b"2"), at: Position { line: 1, column: 20 }, layer: None },
            ])
        })
    );
}
