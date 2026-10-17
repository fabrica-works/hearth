use super::fixtures::{self, Roots, composition::at, expr, guard, key, name, op, port_is, requires};
use crate::{
    entries,
    errors::{Guard, Kind},
    evaluate::{Missing, Op},
    limits::SHIPPED,
    revision::declaration::Condition,
};
use skein_lib::bytes::copy_of;
#[test]
fn a_true_guard_passes_and_a_false_one_errs_at_its_key_with_its_message() {
    let declaration = requires([
        guard(b"ssh", at(3, 5), port_is(22, at(3, 14)), b"ssh must listen on 22"),
        guard(b"web", at(4, 5), port_is(2222, at(4, 14)), b"web needs 2222"),
    ]);
    assert_eq!(
        entries::guards(&declaration, &Roots::x1_prod().context(), &SHIPPED).as_ref(),
        &[entries::Fault {
            kind: Kind::Guard(Guard { name: copy_of(b"ssh"), message: copy_of(b"ssh must listen on 22") }),
            at: at(3, 5),
            also: None
        }]
    );
}
#[test]
fn every_false_guard_of_every_present_task_errs() {
    let (mut side, checked) = fixtures::guarded_tasks();
    crate::call::guards::check(&side.documents, &mut side.errors, &SHIPPED, &fixtures::meta(), &checked);
    let (errors, _) = side.errors.finish();
    let mut names = skein_lib::List::with_capacity(u32::try_from(errors.len()).expect("fault count"));
    for error in &errors {
        names.push(error.task.as_deref()).expect("fault count");
    }
    assert_eq!(names.as_slice(), &[Some(b"core".as_slice()), Some(b"motd".as_slice())]);
}
#[test]
fn a_guard_reads_settings_tasks_and_target() {
    let declaration = requires([guard(
        b"context",
        at(3, 5),
        Condition {
            at: at(3, 20),
            expression: expr(
                4,
                [
                    name(0, b"settings", Missing::Error),
                    key(9, b"port", Missing::Error),
                    name(20, b"tasks", Missing::Error),
                    key(26, b"core", Missing::Error),
                    key(31, b"domain", Missing::Error),
                    name(40, b"target", Missing::Error),
                    key(47, b"name", Missing::Error),
                    op(55, Op::List { length: 3 }),
                ],
            ),
        },
        b"context is wrong",
    )]);
    assert_eq!(entries::guards(&declaration, &Roots::x1_prod().context(), &SHIPPED).as_ref(), &[]);
}
#[test]
fn a_guard_that_cannot_be_evaluated_errs_at_its_condition() {
    let mut declaration = fixtures::optional_guard();
    fixtures::only_mut(&mut declaration.requires).when.expression = expr(
        2,
        [
            name(0, b"settings", Missing::Error),
            key(9, b"wifi", Missing::Error),
            op(17, Op::Integer(1)),
            op(14, Op::Equal),
        ],
    );
    assert_eq!(
        entries::guards(&declaration, &Roots::x1_prod().context(), &SHIPPED).as_ref(),
        &[entries::Fault { kind: Kind::Evaluation(crate::evaluate::FaultKind::Undefined), at: at(3, 20), also: None }]
    );
}
#[test]
fn a_guard_over_an_undefined_optional_setting_is_false_not_an_error() {
    assert_eq!(
        entries::guards(&fixtures::optional_guard(), &Roots::x1_prod().context(), &SHIPPED).as_ref(),
        &[entries::Fault {
            kind: Kind::Guard(Guard { name: copy_of(b"wifi"), message: copy_of(b"wifi needed") }),
            at: at(3, 5),
            also: None
        }]
    );
}
