use super::fixtures::{self, collisions as f};
use crate::{
    errors::{Class, Collision, Kind, Place, Subject},
    revision::{
        Position,
        declaration::{Map, Name},
    },
};
use skein_lib::bytes::copy_of;
#[test]
fn two_tasks_giving_one_tree_path_collide_naming_both() {
    assert_eq!(
        f::kinds(&f::tree_twice()).as_ref(),
        &[(
            Kind::Collision(Collision::Given {
                subject: Subject::Tree,
                name: copy_of(b"/etc/motd"),
                first: copy_of(b"banner"),
                second: copy_of(b"motd"),
            }),
            Class::Configuration
        )]
    );
}
#[test]
fn identical_content_in_two_tasks_still_collides() {
    assert_eq!(f::tree_twice().len(), f::ONE_COLLISION);
}
#[test]
fn a_tree_claim_keeps_both_revision_places() {
    let errors = f::tree_twice();
    let error = fixtures::only(&errors);
    assert_eq!(
        (&error.at, &error.also),
        (
            &Some(Place { path: copy_of(b"tasks/banner/task.json"), position: Some(Position { line: 1, column: 12 }) }),
            &Some(Place { path: copy_of(b"tasks/motd/tree/etc/motd"), position: None })
        )
    );
}
#[test]
fn three_tasks_on_one_path_give_two_errors_against_the_first() {
    let errors = f::collide(
        &[
            f::plan_of(b"c", &[(b"/etc/motd", None)]),
            f::plan_of(b"a", &[(b"/etc/motd", None)]),
            f::plan_of(b"b", &[(b"/etc/motd", None)]),
        ],
        &f::no_expansion(),
    );
    assert_eq!(
        f::kinds(&errors).as_ref(),
        &[
            (
                Kind::Collision(Collision::Given {
                    subject: Subject::Tree,
                    name: copy_of(b"/etc/motd"),
                    first: copy_of(b"a"),
                    second: copy_of(b"b")
                }),
                Class::Configuration
            ),
            (
                Kind::Collision(Collision::Given {
                    subject: Subject::Tree,
                    name: copy_of(b"/etc/motd"),
                    first: copy_of(b"a"),
                    second: copy_of(b"c")
                }),
                Class::Configuration
            ),
        ]
    );
}
#[test]
fn a_data_name_in_two_classes_collides() {
    let expansion = fixtures::kinds::account_expansion(&[
        (b"a", Map::Data, &[fixtures::kinds::file(b"app", [fixtures::kinds::text(Name::Path, b"/srv/a")])]),
        (
            b"b",
            Map::Data,
            &[fixtures::kinds::file(
                b"app",
                [fixtures::kinds::text(Name::Path, b"/srv/b"), fixtures::kinds::text(Name::Class, b"cache")],
            )],
        ),
    ]);
    assert_eq!(
        f::kinds(&f::collide(&[], &expansion)).as_ref(),
        &[(
            Kind::Collision(Collision::Given {
                subject: Subject::DataName,
                name: copy_of(b"app"),
                first: copy_of(b"a"),
                second: copy_of(b"b"),
            }),
            Class::Configuration
        )]
    );
}
#[test]
fn a_tree_path_under_a_data_path_is_hidden() {
    let errors =
        f::collide(&[f::plan_of(b"tree", &[(b"/opt/app/seed", None)])], &f::data(b"data", b"app", b"/opt/app"));
    assert_eq!(
        f::kinds(&errors).as_ref(),
        &[(
            Kind::Collision(Collision::Hidden {
                path: copy_of(b"/opt/app/seed"),
                data: copy_of(b"/opt/app"),
                first: copy_of(b"data"),
                second: copy_of(b"tree"),
            }),
            Class::Configuration
        )]
    );
}
#[test]
fn a_secret_at_a_tree_path_collides() {
    let errors = f::collide(&[f::plan_of(b"tree", &[(b"/etc/key", None)])], &f::secret(b"secret", b"/etc/key"));
    assert_eq!(
        f::kinds(&errors).as_ref(),
        &[(
            Kind::Collision(Collision::Given {
                subject: Subject::Tree,
                name: copy_of(b"/etc/key"),
                first: copy_of(b"secret"),
                second: copy_of(b"tree"),
            }),
            Class::Configuration
        )]
    );
}
#[test]
fn a_secret_identity_or_download_under_a_data_path_is_hidden() {
    let expansion = f::joined(&[
        f::secret(b"a", b"/opt/app/a"),
        f::identity(b"b", b"/opt/app/b"),
        f::download(b"c", b"/opt/app/c", b"tar"),
    ]);
    let errors = f::hidden(expansion);
    assert_eq!(
        f::kinds(&errors).as_ref(),
        &[
            (
                Kind::Collision(Collision::Hidden {
                    path: copy_of(b"/opt/app/a"),
                    data: copy_of(b"/opt/app"),
                    first: copy_of(b"z-data"),
                    second: copy_of(b"a")
                }),
                Class::Configuration
            ),
            (
                Kind::Collision(Collision::Hidden {
                    path: copy_of(b"/opt/app/b"),
                    data: copy_of(b"/opt/app"),
                    first: copy_of(b"z-data"),
                    second: copy_of(b"b")
                }),
                Class::Configuration
            ),
            (
                Kind::Collision(Collision::Hidden {
                    path: copy_of(b"/opt/app/c"),
                    data: copy_of(b"/opt/app"),
                    first: copy_of(b"z-data"),
                    second: copy_of(b"c")
                }),
                Class::Configuration
            ),
        ]
    );
}
#[test]
fn a_hidden_claim_is_placed_at_the_data_entry_even_when_its_task_sorts_last() {
    let errors = f::hidden(f::secret(b"a", b"/opt/app/key"));
    assert_eq!(
        fixtures::only(&errors).at,
        Some(Place { path: copy_of(b"tasks/z-data/task.json"), position: Some(fixtures::kinds::FILE_AT) })
    );
}
#[test]
fn a_tar_download_covering_a_tree_file_a_secret_or_an_identity_path_overlaps() {
    let expansion = f::joined(&[
        f::download(b"tar", b"/opt/app", b"tar"),
        f::secret(b"a", b"/opt/app/a"),
        f::identity(b"b", b"/opt/app/b"),
    ]);
    let errors = f::collide(&[f::plan_of(b"tree", &[(b"/opt/app/c", None)])], &expansion);
    assert_eq!(
        f::kinds(&errors).as_ref(),
        &[
            (
                Kind::Collision(Collision::Overlap {
                    subject: Subject::Download,
                    first: copy_of(b"/opt/app"),
                    second: copy_of(b"/opt/app/a")
                }),
                Class::Configuration
            ),
            (
                Kind::Collision(Collision::Overlap {
                    subject: Subject::Download,
                    first: copy_of(b"/opt/app"),
                    second: copy_of(b"/opt/app/b")
                }),
                Class::Configuration
            ),
            (
                Kind::Collision(Collision::Overlap {
                    subject: Subject::Download,
                    first: copy_of(b"/opt/app"),
                    second: copy_of(b"/opt/app/c")
                }),
                Class::Configuration
            ),
        ]
    );
}
#[test]
fn two_data_paths_equal_or_nested_overlap() {
    let errors = f::collide(
        &[],
        &f::joined(&[f::two_data(b"/opt/app", b"/opt/app"), f::data(b"gamma", b"three", b"/opt/app/child")]),
    );
    assert_eq!(
        f::kinds(&errors).as_ref(),
        &[
            (
                Kind::Collision(Collision::Overlap {
                    subject: Subject::DataPath,
                    first: copy_of(b"/opt/app"),
                    second: copy_of(b"/opt/app")
                }),
                Class::Configuration
            ),
            (
                Kind::Collision(Collision::Overlap {
                    subject: Subject::DataPath,
                    first: copy_of(b"/opt/app"),
                    second: copy_of(b"/opt/app/child")
                }),
                Class::Configuration
            ),
        ]
    );
}
#[test]
fn a_secret_and_a_tree_file_of_one_task_collide_naming_it_twice() {
    let errors = f::collide(&[f::plan_of(b"same", &[(b"/etc/key", None)])], &f::secret(b"same", b"/etc/key"));
    assert_eq!(
        f::kinds(&errors).as_ref(),
        &[(
            Kind::Collision(Collision::Given {
                subject: Subject::Tree,
                name: copy_of(b"/etc/key"),
                first: copy_of(b"same"),
                second: copy_of(b"same"),
            }),
            Class::Configuration
        )]
    );
}
#[test]
fn a_download_inside_a_tar_downloads_directory_overlaps() {
    let errors = f::tar_over(f::download(b"file", b"/opt/app/file", b"file"));
    assert_eq!(
        f::kinds(&errors).as_ref(),
        &[(
            Kind::Collision(Collision::Overlap {
                subject: Subject::Download,
                first: copy_of(b"/opt/app"),
                second: copy_of(b"/opt/app/file"),
            }),
            Class::Configuration
        )]
    );
}
#[test]
fn a_file_download_covers_only_itself() {
    let errors =
        f::collide(&[f::plan_of(b"tree", &[(b"/opt/app/child", None)])], &f::download(b"file", b"/opt/app", b"file"));
    assert_eq!(errors.as_ref(), &[]);
}
#[test]
fn a_path_prefix_without_a_slash_does_not_hide_another_path() {
    assert_eq!(
        f::collide(&[f::plan_of(b"tree", &[(b"/opt/app-other", None)])], &f::data(b"data", b"app", b"/opt/app"))
            .as_ref(),
        &[]
    );
}
#[test]
fn a_full_claim_table_reports_a_limit_at_the_entry() {
    let errors = f::collide_with_limits(&[], &f::data(b"data", b"app", b"/opt/app"), &f::EMPTY_CLAIMS);
    assert_eq!(
        f::kinds(&errors).as_ref(),
        &[(Kind::Evaluation(crate::evaluate::FaultKind::Limit), Class::Configuration)]
    );
}
#[test]
fn two_secret_names_collide_even_when_their_paths_differ() {
    let expansion = f::joined(&[f::secret(b"a", b"/etc/a"), f::secret(b"b", b"/etc/b")]);
    assert_eq!(
        f::kinds(&f::collide(&[], &expansion)).as_ref(),
        &[(
            Kind::Collision(Collision::Given {
                subject: Subject::Secret,
                name: copy_of(b"mail/tls.key"),
                first: copy_of(b"a"),
                second: copy_of(b"b"),
            }),
            Class::Configuration
        )]
    );
}
#[test]
fn two_identity_names_collide_even_when_their_paths_differ() {
    let expansion = f::joined(&[f::identity(b"a", b"/etc/a"), f::identity(b"b", b"/etc/b")]);
    assert_eq!(
        f::kinds(&f::collide(&[], &expansion)).as_ref(),
        &[(
            Kind::Collision(Collision::Given {
                subject: Subject::Identity,
                name: copy_of(b"ssh"),
                first: copy_of(b"a"),
                second: copy_of(b"b"),
            }),
            Class::Configuration
        )]
    );
}
#[test]
fn an_identity_public_path_collides_with_a_tree_path() {
    let expansion = f::identity_with_public(b"identity", b"/etc/private", b"/etc/public");
    let errors = f::collide(&[f::plan_of(b"tree", &[(b"/etc/public", None)])], &expansion);
    assert_eq!(
        f::kinds(&errors).as_ref(),
        &[(
            Kind::Collision(Collision::Given {
                subject: Subject::Tree,
                name: copy_of(b"/etc/public"),
                first: copy_of(b"identity"),
                second: copy_of(b"tree"),
            }),
            Class::Configuration
        )]
    );
}
#[test]
fn an_identitys_public_and_private_paths_collide_when_equal() {
    let errors = f::collide(&[], &f::identity_with_public(b"same", b"/etc/key", b"/etc/key"));
    assert_eq!(
        f::kinds(&errors).as_ref(),
        &[(
            Kind::Collision(Collision::Given {
                subject: Subject::Identity,
                name: copy_of(b"/etc/key"),
                first: copy_of(b"same"),
                second: copy_of(b"same"),
            }),
            Class::Configuration
        )]
    );
}

mod declarations;
