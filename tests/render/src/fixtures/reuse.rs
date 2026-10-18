//! Two revisions with all successful blob interpretations in common.
use super::{Repository, document, entry, oid, set_document, task_patch, with_int_setting};
use hearth_domain_render::{
    errors::Error,
    limits::{Limits, SHIPPED},
    revision::{Kind, Mode},
    vocabulary::{Answer, As, Decoded, Request},
};
// The diff and reference render use the same stable schedule.
pub const SEED: u64 = 0x28_57;
const NEW_DOCUMENT: u8 = b'n';
const REFUSED_DOCUMENT: u8 = b'r';
const CHANGED_DECLARATION: u8 = b'c';
const FILES: &[(&[u8], u8, Mode, &[u8])] = &[
    (b"tasks/motd/tree/etc/greeting.j2", b't', Mode::File, b"{{ settings.x }}"),
    (b"tasks/motd/tree/etc/static", b's', Mode::File, b"static\n"),
    (b"tasks/motd/tree/etc/link", b'l', Mode::Link, b"static"),
];
#[must_use]
pub fn repository(refused: bool) -> Repository {
    let mut repository = with_int_setting();
    set_document(&mut repository, b'2', task_patch(b"motd", b"x", Kind::Int(7)));
    for (path, byte, mode, text) in FILES {
        let mut file = entry(path, *byte);
        file.mode = *mode;
        file.size = text.len() as u64;
        repository.entries.push(file);
        repository.blobs.push((oid(*byte), (*text).into()));
    }
    if refused {
        repository.entries.push(entry(b"envs/broken.json", REFUSED_DOCUMENT));
        repository.documents.push((oid(REFUSED_DOCUMENT), document(None)));
        super::refuse_document(&mut repository, REFUSED_DOCUMENT);
    }
    super::sort_entries(&mut repository);
    repository
}
#[must_use]
pub fn changed_document(from: &Repository) -> Repository {
    edited(from, Edit::Document)
}
#[must_use]
pub fn moved_and_changed(from: &Repository) -> Repository {
    edited(from, Edit::Moved)
}
fn move_paths(to: &mut Repository) {
    for entry in &mut to.entries {
        match entry.path.as_ref() {
            b"envs/broken.json" => entry.path = b"envs/renamed.json".as_slice().into(),
            b"tasks/motd/tree/etc/greeting.j2" => entry.path = b"tasks/motd/tree/etc/renamed.j2".as_slice().into(),
            b"tasks/motd/tree/etc/static" => entry.path = b"tasks/motd/tree/etc/moved".as_slice().into(),
            b"tasks/motd/task.json" => entry.oid = oid(CHANGED_DECLARATION),
            _ => {}
        }
    }
    let (_, declaration) = to.documents.iter().find(|(id, _)| *id == oid(b'4')).expect("declaration");
    let Decoded::Declaration(mut declaration) = declaration.clone() else { panic!("declaration") };
    declaration.settings = Box::new([]);
    to.documents.push((oid(CHANGED_DECLARATION), Decoded::Declaration(declaration)));
}
#[must_use]
pub fn second_reads(run: &crate::parent::Run) -> Vec<(&[u8], As)> {
    run.requests
        .iter()
        .filter_map(|request| match request {
            Request::Read { commit, blobs, .. } if *commit == oid(b'b') => Some(blobs),
            Request::Record { .. }
            | Request::Resolve { .. }
            | Request::Find { .. }
            | Request::Keep { .. }
            | Request::Hunks { .. }
            | Request::List { .. }
            | Request::Read { .. } => None,
        })
        .flatten()
        .map(|blob| (blob.oid.0.as_ref(), blob.kind))
        .collect()
}
#[must_use]
pub fn errors(run: &crate::parent::Run, side: Option<hearth_domain_render::diff::Side>) -> Vec<Error> {
    run.answers
        .iter()
        .filter_map(|answer| match answer {
            Answer::Error(error) if error.side == side => {
                let mut error = error.clone();
                error.side = None;
                Some(error)
            }
            Answer::Spec(_) | Answer::Error(_) | Answer::Change(_) | Answer::Rendered { .. } => None,
        })
        .collect()
}
#[must_use]
pub fn kept_templates() -> Repository {
    let mut repository = super::decoded_comments_repository();
    repository
        .entries
        .retain(|entry| entry.oid != oid(u8::try_from(super::COMMENT_TEMPLATES - 1).expect("sixteen templates")));
    for entry in &repository.entries {
        if entry.path.ends_with(b".j2") {
            repository.documents.push((entry.oid.clone(), Decoded::Template(super::comment_template())));
        }
    }
    repository
}
#[must_use]
pub fn bound() -> u64 {
    hearth_domain_render::limits::worst_case(&SHIPPED).expect("shipped bound")
}
pub const LIMITS: Limits = SHIPPED;

pub const NEW_OID: &[u8] = b"nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn";
pub const REFUSED_OID: &[u8] = b"rrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrr";

#[must_use]
pub fn error_paths(errors: &[Error]) -> Vec<&[u8]> {
    errors.iter().map(|error| error.at.as_ref().expect("placed error").path.as_ref()).collect()
}
#[derive(Clone, Copy, Debug)]
pub enum Interpretation {
    StaticTemplate,
    TemplateStatic,
    LinkStatic,
}
#[must_use]
pub fn reinterpreted(from: &Repository, interpretation: Interpretation) -> Repository {
    edited(from, Edit::Interpretation(interpretation))
}
fn reinterpret(to: &mut Repository, interpretation: Interpretation) {
    let (source, destination, mode): (&[u8], &[u8], Mode) = match interpretation {
        Interpretation::StaticTemplate => (b"tasks/motd/tree/etc/static", b"tasks/motd/tree/etc/static.j2", Mode::File),
        Interpretation::TemplateStatic => {
            (b"tasks/motd/tree/etc/greeting.j2", b"tasks/motd/tree/etc/greeting", Mode::File)
        }
        Interpretation::LinkStatic => (b"tasks/motd/tree/etc/link", b"tasks/motd/tree/etc/link", Mode::File),
    };
    let entry = to.entries.iter_mut().find(|entry| entry.path.as_ref() == source).expect("file interpretation");
    entry.path = destination.into();
    entry.mode = mode;
}
pub const STATIC_OID: &[u8] = b"ssssssssssssssssssssssssssssssssssssssss";
pub const TEMPLATE_OID: &[u8] = b"tttttttttttttttttttttttttttttttttttttttt";
pub const LINK_OID: &[u8] = b"llllllllllllllllllllllllllllllllllllllll";

#[derive(Clone, Copy)]
enum Edit {
    Document,
    Moved,
    Interpretation(Interpretation),
}
fn edited(from: &Repository, edit: Edit) -> Repository {
    let mut to = from.clone();
    match edit {
        Edit::Document => {
            to.entries.push(entry(b"envs/new.json", NEW_DOCUMENT));
            to.documents.push((oid(NEW_DOCUMENT), document(None)));
        }
        Edit::Moved => move_paths(&mut to),
        Edit::Interpretation(interpretation) => reinterpret(&mut to, interpretation),
    }
    super::sort_entries(&mut to);
    to
}
