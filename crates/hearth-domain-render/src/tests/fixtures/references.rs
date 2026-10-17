//! Shared reference-checking inputs.
use super::{composition::at, declaration, declared_spec, read};
use crate::{
    evaluate::{Expression, Operation},
    revision::declaration::{Declaration, Text},
};
use skein_lib::bytes;
#[must_use]
pub fn declared_for_references() -> Declaration {
    let core = read(b"core", &[Text { at: at(1, 1), text: bytes::copy_of(b"domain") }]);
    let mut mail = read(b"mail", &[]);
    mail.exports = true;
    let mut wifi = read(b"wifi", &[]);
    wifi.optional = true;
    declaration(&[declared_spec(b"greeting"), declared_spec(b"port")], &[core, mail, wifi])
}
#[must_use]
pub fn expression<const N: usize>(operations: [Operation; N]) -> Expression {
    super::expr(u32::try_from(N).expect("fixture fits"), operations)
}
#[must_use]
pub fn reference_instructions(expression: Expression) -> [crate::evaluate::template::Does; 4] {
    use crate::evaluate::template::{Does, Names};
    [
        Does::Output(expression.clone()),
        Does::Branch { condition: expression.clone(), over: 0 },
        Does::Enter { names: Names::One { name: bytes::copy_of(b"x") }, iterable: expression.clone(), over: 0 },
        Does::Set { name: bytes::copy_of(b"x"), value: expression },
    ]
}
pub const REFERENCE_OFFSET: u32 = 21;
#[must_use]
pub fn reference_typo() -> Expression {
    expression([
        super::name(12, b"settings", crate::evaluate::Missing::Error),
        super::key(REFERENCE_OFFSET, b"greting", crate::evaluate::Missing::Error),
    ])
}
#[derive(Clone, Copy, Debug)]
pub enum ReferencePlace {
    Guard,
    If,
    ForEach,
    Key,
    Text,
    Typed,
    Strings,
    Range,
}
#[must_use]
pub fn reference_declaration(place: ReferencePlace) -> Declaration {
    use super::{entries, entry, field, fields, guard, plain, range_field, requires, setting_tag};
    use crate::revision::declaration::{Literal, Name};
    use alloc::boxed::Box;
    match place {
        ReferencePlace::Guard => requires([guard(b"known", at(3, 5), super::setting(b"greting"), b"set one")]),
        ReferencePlace::If => entries([entry(plain(b"ops"), None, Some(super::setting(b"greting")))]),
        ReferencePlace::ForEach => entries([entry(plain(b"ops"), Some(super::setting(b"greting")), None)]),
        ReferencePlace::Key => entries([entry(setting_tag(b"greting"), None, None)]),
        ReferencePlace::Text => fields([field(Name::From, Literal::Text(setting_tag(b"greting")))]),
        ReferencePlace::Typed => fields([field(Name::Uid, Literal::Typed(setting_tag(b"greting")))]),
        ReferencePlace::Strings => fields([field(
            Name::Groups,
            Literal::Strings { at: super::FIELD_AT, each: Box::new([setting_tag(b"greting")]) },
        )]),
        ReferencePlace::Range => fields([range_field(Literal::Typed(setting_tag(b"greting")))]),
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Reference {
    pub root: &'static [u8],
    pub task: &'static [u8],
    pub setting: Option<&'static [u8]>,
}
pub type ReferenceFields<'a> = (&'a [u8], crate::errors::Root, &'a [u8], Option<&'a [u8]>, u32);
#[must_use]
pub fn reference_expression(reference: Reference) -> Expression {
    use super::{key, name};
    use crate::evaluate::Missing;
    if let Some(setting) = reference.setting {
        expression([
            name(0, reference.root, Missing::Error),
            key(6, reference.task, Missing::Error),
            key(11, setting, Missing::Error),
        ])
    } else {
        expression([name(0, reference.root, Missing::Error), key(6, reference.task, Missing::Error)])
    }
}
pub type ReferenceFault = Option<(crate::errors::Schema, u32)>;

#[must_use]
pub fn check_reference(expression: &Expression) -> ReferenceFault {
    crate::schema::references::check(expression, &declared_for_references())
}
#[must_use]
pub fn template_references(template: &crate::evaluate::template::Template) -> ReferenceFault {
    crate::schema::references::template(template, &declared_for_references())
}
