use super::{Name, fixture, fixtures};
use crate::{
    errors::{Kind, Repository, Schema},
    kinds::files::Make,
    revision::Mode,
};
use fixture::{NAMED_TEMPLATE, planned_sources, source_declaration, source_entry, source_listing};
#[test]
fn a_template_no_entry_names_is_unused() {
    let errors = planned_sources(&fixtures::declaration(&[], &[]), &[source_listing(NAMED_TEMPLATE, Mode::File, 20)])
        .expect_err("unused template");
    assert_eq!(fixtures::only(&errors).kind, Kind::Repository(Repository::Unused));
}
#[test]
fn a_template_field_with_a_tag_is_not_interpolated_and_names_no_template() {
    let declared = source_declaration(source_entry(Name::Template, b"templates/{{ value }}.j2"));
    let errors = planned_sources(&declared, &[]).expect_err("literal template name");
    assert_eq!(fixtures::only(&errors).kind, Kind::Repository(Repository::NoTemplate));
}
#[test]
fn a_template_entry_naming_no_template_is_a_repository_error() {
    let declared = source_declaration(source_entry(Name::Template, b"templates/missing.j2"));
    let errors = planned_sources(&declared, &[]).expect_err("missing template");
    assert_eq!(fixtures::only(&errors).kind, Kind::Repository(Repository::NoTemplate));
}
#[test]
fn a_copy_of_an_unlisted_file_is_a_missing_file() {
    let declared = fixture::copy_declaration();
    let errors = planned_sources(&declared, &[]).expect_err("missing copy");
    assert_eq!(fixtures::only(&errors).kind, Kind::Schema(Schema::File { path: b"lab-ca/lab.crt".as_slice().into() }));
}
#[test]
fn a_copy_over_a_tree_path_is_twice() {
    let errors = planned_sources(
        &fixture::copy_declaration(),
        &[source_listing(b"tasks/motd/tree/etc/greeting", Mode::File, 20)],
    )
    .expect_err("two sources");
    assert_eq!(fixtures::only(&errors).kind, Kind::Repository(Repository::Destination));
}
#[test]
fn a_template_entry_over_a_tree_path_is_twice() {
    let errors = planned_sources(
        &source_declaration(source_entry(Name::Template, b"templates/greeting.j2")),
        &[source_listing(b"tasks/motd/tree/etc/greeting", Mode::File, 20)],
    )
    .expect_err("two sources");
    assert_eq!(fixtures::only(&errors).kind, Kind::Repository(Repository::Destination));
}
#[test]
fn a_template_only_a_dropped_entry_names_is_still_used() {
    let plans =
        planned_sources(&fixture::dropped_template_declaration(), &[source_listing(NAMED_TEMPLATE, Mode::File, 20)])
            .expect("named by dropped entry");
    assert_eq!(fixtures::only(&plans).installs.as_ref(), []);
}
const COPY_MODES: &[(Option<&[u8]>, u16)] = &[(None, 0o644), (Some(b"0600"), 0o600)];
#[test]
fn a_copy_of_an_executable_file_is_0644_unless_given() {
    for (row, &(mode, expected)) in COPY_MODES.iter().enumerate() {
        let mut entry = source_entry(Name::Copy, b"lab-ca/lab.crt");
        if let Some(mode) = mode {
            entry.fields = Box::new([
                fixtures::only(&entry.fields).clone(),
                fixtures::field(
                    Name::Mode,
                    crate::revision::declaration::Literal::Plain { at: fixture::FIELD_AT, text: mode.into() },
                ),
            ]);
        }
        let plans =
            planned_sources(&source_declaration(entry), &fixtures::forms::listing(Mode::Executable, 20)).expect("copy");
        assert_eq!(
            fixtures::only(&fixtures::only(&plans).installs).make,
            Make::Copied { source: b"lab-ca/lab.crt".as_slice().into(), mode: expected },
            "row {row}"
        );
    }
}
#[test]
fn a_template_entry_runs_once_per_element_with_its_key_and_value() {
    let keeps = fixture::source_keeps(&fixture::element_declaration(), fixture::element_template());
    assert_eq!(
        keeps.as_ref(),
        [(NAMED_TEMPLATE.into(), b"0:sudo".as_slice().into()), (NAMED_TEMPLATE.into(), b"1:wheel".as_slice().into())]
    );
}
#[test]
fn a_dropped_template_entry_is_never_run() {
    assert_eq!(
        fixture::source_keeps(&fixture::dropped_template_declaration(), fixture::element_template()).as_ref(),
        []
    );
}
#[test]
fn a_failed_copy_read_ends_the_render_with_try_again() {
    use crate::vocabulary::{Answer, Class, Event, Failure, Outcome};
    let (mut render, env, mut out) = fixture::copy_render();
    crate::resume(&mut render, &env, &mut out);
    drop(out.pop());
    crate::step(
        &mut render,
        &env,
        Event::Failed {
            request: skein_lib::Token::new(2),
            class: Class::Operational,
            at: None,
            failure: Failure::Store,
        },
        &mut out,
    );
    assert_eq!(
        fixtures::finish_render(&mut render, &env, &mut out).as_ref(),
        [Answer::Rendered { outcome: Outcome::TryAgain, diff: None, left_out: 0 }]
    );
}
#[test]
fn a_copy_is_requested_once_as_cached_for_all_elements() {
    use crate::vocabulary::{As, Blob, Output, Request};
    let (mut render, env, mut out) = fixture::copy_render();
    crate::resume(&mut render, &env, &mut out);
    assert_eq!(
        out.pop(),
        Some(Output::Request(Request::Read {
            request: skein_lib::Token::new(2),
            repository: b"repository".as_slice().into(),
            commit: crate::revision::Oid(b"commit".as_slice().into()),
            blobs: Box::new([Blob {
                oid: crate::revision::Oid(b"file".as_slice().into()),
                size: 20,
                kind: As::Cached,
            }]),
        }))
    );
}
#[test]
fn a_copy_larger_than_the_static_file_limit_is_large() {
    let errors = planned_sources(
        &fixture::copy_declaration(),
        &fixtures::forms::listing(Mode::File, u64::from(crate::limits::SHIPPED.static_file) + 1),
    )
    .expect_err("large copy");
    assert_eq!(
        fixtures::only(&errors).kind,
        Kind::Schema(Schema::Large {
            path: b"lab-ca/lab.crt".as_slice().into(),
            max: u64::from(crate::limits::SHIPPED.static_file),
        })
    );
}
