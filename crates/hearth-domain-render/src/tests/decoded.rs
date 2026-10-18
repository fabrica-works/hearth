use super::fixtures;

struct TemplateBytes {
    text: &'static [u8],
    name: &'static [u8],
    bytes: u64,
}
// Two instructions, one operation, and both occurrences of the name (2.3).
const TEMPLATES: &[TemplateBytes] = &[
    TemplateBytes { text: b"abc", name: b"x", bytes: 101 },
    TemplateBytes { text: b"abcdef", name: b"xyz", bytes: 108 },
];
#[test]
fn decoded_templates_count_text_names_instructions_and_operations() {
    for row in TEMPLATES {
        let template = fixtures::charged_template(row.text, row.name);
        assert_eq!(crate::revision::held::template::template(&template), row.bytes, "name {:?}", row.name);
    }
}

#[test]
fn a_document_of_list_elements_counts_32_bytes_each() {
    assert_eq!(
        crate::revision::held::document(&fixtures::list_document(), &crate::limits::SHIPPED),
        fixtures::DOCUMENT_HELD_BYTES
    );
}
#[test]
fn four_documents_fit_the_decoded_revision_budget() {
    let call = fixtures::decoded_documents(fixtures::ADMITTED_DOCUMENTS);
    assert_eq!(call.outcome, crate::vocabulary::Outcome::Done);
    assert_eq!(fixtures::kept(&call).as_ref(), []);
}
#[test]
fn a_fifth_document_fails_the_render_at_its_path() {
    let call = fixtures::decoded_documents(fixtures::OVER_BUDGET_DOCUMENTS);
    assert_eq!(call.outcome, crate::vocabulary::Outcome::Failed);
    assert_eq!(
        fixtures::kept(&call).as_ref(),
        &[fixtures::decode_error(
            b"envs/e.json",
            None,
            crate::errors::Decode::TooLarge {
                limit: crate::errors::Limit::Decoded,
                max: crate::limits::SHIPPED.decoded
            },
            None,
        )]
    );
}
