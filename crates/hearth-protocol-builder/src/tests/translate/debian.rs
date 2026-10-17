use crate::{
    config::{self, document},
    render::revision,
    tests::fixtures,
};
#[test]
fn a_documents_debian_translates_to_render_and_back() {
    for (row, input) in fixtures::debian_documents().into_iter().enumerate() {
        let translated = crate::translate::documents::document(input.clone());
        let back = document::Document {
            at: position(translated.at),
            members: None,
            layer: document::Layer {
                tasks: Box::new([]),
                placement: None,
                debian: Some(value(translated.layer.debian.expect("root"))),
            },
        };
        assert_eq!(back, input, "row {row}: Debian root retains each field and position");
    }
}
fn position(at: revision::Position) -> config::Position {
    config::Position { offset: 0, line: at.line, column: at.column }
}
fn value(value: revision::Value) -> config::json::Value {
    let revision::Kind::Map(pairs) = value.kind else { panic!("root object") };
    let mut fields = skein_lib::List::with_capacity(u32::try_from(pairs.len()).expect("fixture count"));
    for pair in pairs {
        let revision::Kind::String(text) = pair.value.kind else { panic!("suite") };
        fields
            .push(config::json::Entry {
                key: pair.key,
                at: position(pair.at),
                value: config::json::Value { at: position(pair.value.at), kind: config::json::Kind::String(text) },
            })
            .expect("fixture count");
    }
    config::json::Value { at: position(value.at), kind: config::json::Kind::Map(fields.into_boxed()) }
}
