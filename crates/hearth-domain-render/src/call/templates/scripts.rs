//! Cache customization templates as script content, with no tree destination.
use super::{Class, Digest, Error, Keeping, Kind, Outcome, Place, Request, Side, Target, Token, parts, template};
use alloc::boxed::Box;
impl Target {
    pub(super) fn render(
        &mut self,
        side: &mut Side,
        path: &[u8],
        task: &[u8],
        via: Option<&crate::kinds::files::Via>,
    ) -> Option<(Digest, Box<[u8]>)> {
        let oid = side.listed_entry(path).expect("planned template listed").oid.clone();
        let template = side.templates.get(&oid)?;
        let context = self.roots.as_ref().expect("task roots").context();
        let element = super::element(&self.expansion, &side.documents, task, via, &context, &side.limits);
        match template::run(
            template,
            &context,
            element,
            &side.limits,
            side.limits.rendered_target.saturating_sub(self.rendered),
        ) {
            Ok(bytes) => {
                let length = u64::try_from(bytes.len()).expect("file bound");
                self.rendered = self.rendered.checked_add(length).expect("target bound");
                let digest = Digest(skein_lib::sha256::digest(&bytes));
                side.sizes.insert(digest, length).expect("rendered bound");
                Some((digest, bytes))
            }
            Err(fault) => {
                self.outcome = self.outcome.worst(Outcome::Failed);
                side.errors.push(Error {
                    side: None,
                    kind: Kind::Evaluation(fault.kind),
                    class: Class::Configuration,
                    at: Some(Place {
                        path: skein_lib::bytes::copy_of(path),
                        position: Some(template::place(&template.lines, fault.at)),
                    }),
                    also: None,
                    target: Some(self.meta.target.clone()),
                    task: Some(skein_lib::bytes::copy_of(task)),
                });
                None
            }
        }
    }
    pub(super) fn script(&mut self, side: &mut Side, task: &[u8]) -> bool {
        let mut source = None;
        for script in &self.scripts {
            if script.task.as_ref() == task {
                source = Some(script.clone());
            }
        }
        let Some(source) = source else { return false };
        if !source.path.ends_with(b".j2") {
            let oid = &side.listed_entry(&source.path).expect("script listed").oid;
            let script = parts::config::Script {
                task: source.task,
                order: source.order,
                content_digest: *side.cached.get(oid).expect("script cached"),
            };
            self.script_rows.insert((script.order, script.task.clone()), script).expect("task bound");
            return false;
        }
        if let Some((digest, bytes)) = self.render(side, &source.path, task, None) {
            self.keeping = Some(Keeping::Script(parts::config::Script {
                task: source.task,
                order: source.order,
                content_digest: digest,
            }));
            self.request = Some(Request::Keep { request: Token::new(4), path: source.path, sha256: digest, bytes });
            side.ready = false;
        }
        true
    }
}
