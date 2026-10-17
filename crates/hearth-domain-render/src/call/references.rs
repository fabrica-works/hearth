//! Check each held template once for each task path, before target evaluation.
use super::Side;
use crate::{
    errors::{Error, Kind, Place, Schema},
    evaluate::template::{self, Template},
    revision::{self, Oid},
    schema::references,
    vocabulary::{Class, Outcome},
};
impl Side {
    pub(super) fn check_template(&mut self, oid: &Oid, template: &Template) {
        for index in 0..self.entries.len() {
            let entry = self.entries.get(index).expect("listing cursor");
            let revision::Class::Tree { task, path } = revision::class(&entry.path) else { continue };
            if &entry.oid != oid || !path.ends_with(b".j2") || !self.needs(task) {
                continue;
            }
            let Some((_, Some(declaration))) = self.documents.declaration(task) else { continue };
            if let Some((mut schema, at)) = references::template(template, declaration) {
                if let Schema::Unlisted { reader, root: _, task: _, setting: _ } = &mut schema {
                    *reader = skein_lib::bytes::copy_of(task);
                }
                self.errors.push(Error {
                    side: None,
                    kind: Kind::Schema(schema),
                    class: Class::Configuration,
                    at: Some(Place { path: entry.path.clone(), position: Some(template::place(&template.lines, at)) }),
                    also: None,
                    target: None,
                    task: Some(skein_lib::bytes::copy_of(task)),
                });
                *self.invalid.get_mut(index).expect("listing flags") = true;
                self.outcome = self.outcome.worst(Outcome::Failed);
            }
        }
    }
}
