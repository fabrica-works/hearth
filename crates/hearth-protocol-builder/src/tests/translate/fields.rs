use crate::{config, render::revision::declaration as target, tests::fixtures, translate::declarations};
fn name_back(name: target::Name) -> config::entries::Name {
    match name {
        target::Name::Action => config::entries::Name::Action,
        target::Name::Age => config::entries::Name::Age,
        target::Name::Algorithm => config::entries::Name::Algorithm,
        target::Name::Argument => config::entries::Name::Argument,
        target::Name::Base => config::entries::Name::Base,
        target::Name::Class => config::entries::Name::Class,
        target::Name::Component => config::entries::Name::Component,
        target::Name::Copy => config::entries::Name::Copy,
        target::Name::Count => config::entries::Name::Count,
        target::Name::Directory => config::entries::Name::Directory,
        target::Name::From => config::entries::Name::From,
        target::Name::Gecos => config::entries::Name::Gecos,
        target::Name::Gid => config::entries::Name::Gid,
        target::Name::Group => config::entries::Name::Group,
        target::Name::Groups => config::entries::Name::Groups,
        target::Name::Home => config::entries::Name::Home,
        target::Name::Kind => config::entries::Name::Kind,
        target::Name::Link => config::entries::Name::Link,
        target::Name::Mode => config::entries::Name::Mode,
        target::Name::Order => config::entries::Name::Order,
        target::Name::Owner => config::entries::Name::Owner,
        target::Name::Param => config::entries::Name::Param,
        target::Name::Password => config::entries::Name::Password,
        target::Name::Path => config::entries::Name::Path,
        target::Name::Paths => config::entries::Name::Paths,
        target::Name::PublicPath => config::entries::Name::PublicPath,
        target::Name::Sha256 => config::entries::Name::Sha256,
        target::Name::Shell => config::entries::Name::Shell,
        target::Name::Strip => config::entries::Name::Strip,
        target::Name::Subordinate => config::entries::Name::Subordinate,
        target::Name::Template => config::entries::Name::Template,
        target::Name::Type => config::entries::Name::Type,
        target::Name::Uid => config::entries::Name::Uid,
        target::Name::Unit => config::entries::Name::Unit,
        target::Name::Unpack => config::entries::Name::Unpack,
        target::Name::Url => config::entries::Name::Url,
        target::Name::Value => config::entries::Name::Value,
    }
}
fn position_back(at: crate::render::revision::Position) -> config::Position {
    config::Position { offset: 0, line: at.line, column: at.column }
}
fn text_back(text: target::Interpolated) -> config::declaration::Text {
    let target::Interpolated::Plain { at, text } = text else { panic!("plain fixture") };
    config::declaration::Text { at: position_back(at), text }
}
fn literal_back(literal: target::Literal) -> config::entries::Literal {
    use config::entries::Literal;
    match literal {
        target::Literal::Text(text) => Literal::Text(text_back(text)),
        target::Literal::Plain { at, text } => {
            Literal::Plain(config::declaration::Text { at: position_back(at), text })
        }
        target::Literal::Typed(text) => Literal::Typed(text_back(text)),
        target::Literal::Int(target::Number { at, value }) => {
            Literal::Int(config::declaration::Number { at: position_back(at), value })
        }
        target::Literal::Bool { at, value } => Literal::Bool { at: position_back(at), value },
        target::Literal::Strings { at, each } => Literal::Strings { at: position_back(at), each: texts_back(each) },
        target::Literal::Range { at, fields } => Literal::Range { at: position_back(at), fields: fields_back(fields) },
    }
}
fn field_back(field: target::Field) -> config::entries::Field {
    let target::Field { name, at, value } = field;
    config::entries::Field { name: name_back(name), at: position_back(at), value: literal_back(value) }
}
#[test]
fn each_name_translates_to_render_and_back() {
    for (row, &input) in fixtures::FIELD_NAMES.iter().enumerate() {
        let back = name_back(declarations::name(input));
        assert_eq!(back, input, "row {row}: field name");
    }
}
#[test]
fn each_literal_translates_to_render_and_back() {
    for (row, input) in fixtures::every_literal().into_iter().enumerate() {
        let translated = declarations::literal(input.clone(), &crate::limits::SHIPPED.template).expect("literal");
        let back = literal_back(translated);
        assert_eq!(back, input, "row {row}: literal");
    }
}
#[test]
fn a_template_field_with_a_tag_is_kept_as_written() {
    let declared = fixtures::read_declaration(br#"{"files":{"/etc/issue":{"template":"{{ key }}.j2"}}}"#);
    let field = fixtures::only(&fixtures::only(&fixtures::only(&declared.entries).entries).fields);
    assert_eq!(
        field.value,
        target::Literal::Plain {
            at: crate::render::revision::Position { line: 1, column: 36 },
            text: skein_lib::bytes::copy_of(b"{{ key }}.j2")
        }
    );
}

fn texts_back(each: Box<[target::Interpolated]>) -> Box<[config::declaration::Text]> {
    let mut texts = skein_lib::List::with_capacity(u32::try_from(each.len()).expect("texts"));
    for text in each {
        texts.push(text_back(text)).expect("texts");
    }
    texts.into_boxed()
}
fn fields_back(each: Box<[target::Field]>) -> Box<[config::entries::Field]> {
    let mut fields = skein_lib::List::with_capacity(u32::try_from(each.len()).expect("fields"));
    for field in each {
        fields.push(field_back(field)).expect("fields");
    }
    fields.into_boxed()
}
