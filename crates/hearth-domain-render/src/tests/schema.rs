use super::fixtures::{self, composition::at};
use crate::{errors::Schema, revision::declaration::Text, schema::Fault};
use skein_lib::bytes::copy_of;

#[test]
fn a_read_of_a_setting_its_task_lacks_is_refused_at_it() {
    let core = fixtures::declaration(&[fixtures::declared_spec(b"domain")], &[]);
    let motd =
        fixtures::declaration(&[], &[fixtures::read(b"core", &[Text { at: at(1, 34), text: copy_of(b"domian") }])]);
    let documents = fixtures::declarations(&[(b"core", Some(core)), (b"motd", Some(motd))]);
    assert_eq!(
        fixtures::check_declarations(&documents).as_ref(),
        &[(
            copy_of(b"motd"),
            Fault {
                schema: Schema::Unread {
                    task: copy_of(b"core"),
                    setting: copy_of(b"domian"),
                    declared: Box::new([copy_of(b"domain")])
                },
                document: 1,
                at: at(1, 34)
            }
        )],
        "format.md, 6.5: the reader's, at its setting"
    );
}
#[test]
fn a_read_of_a_task_tasks_lacks_is_refused_at_its_key() {
    let motd = fixtures::declaration(&[], &[fixtures::read(b"mail", &[])]);
    let documents = fixtures::declarations(&[(b"motd", Some(motd))]);
    assert_eq!(
        fixtures::check_declarations(&documents).as_ref(),
        &[(copy_of(b"motd"), Fault { schema: Schema::Task { task: copy_of(b"mail") }, document: 0, at: at(1, 12) })]
    );
}
const READ_FLAGS: &[(bool, bool)] = &[(true, false), (false, true)];
#[test]
fn an_optional_or_exporting_read_of_a_task_tasks_lacks_is_still_refused() {
    for (row, &(optional, exports)) in READ_FLAGS.iter().enumerate() {
        let read = crate::revision::declaration::Read { exports, optional, ..fixtures::read(b"mail", &[]) };
        let documents = fixtures::declarations(&[(b"motd", Some(fixtures::declaration(&[], &[read])))]);
        assert_eq!(
            fixtures::check_declarations(&documents).as_ref(),
            &[(
                copy_of(b"motd"),
                Fault { schema: Schema::Task { task: copy_of(b"mail") }, document: 0, at: at(1, 12) }
            )],
            "row {row}"
        );
    }
}
#[test]
fn a_read_of_a_refused_task_is_not_checked() {
    let motd =
        fixtures::declaration(&[], &[fixtures::read(b"core", &[Text { at: at(1, 34), text: copy_of(b"domian") }])]);
    let documents = fixtures::declarations(&[(b"core", None), (b"motd", Some(motd))]);
    assert_eq!(fixtures::check_declarations(&documents).as_ref(), &[]);
}
#[test]
fn a_read_fault_makes_only_its_reading_task_unsound() {
    let motd = fixtures::declaration(&[], &[fixtures::read(b"mail", &[])]);
    let documents = fixtures::declarations(&[(b"core", Some(fixtures::declaration(&[], &[]))), (b"motd", Some(motd))]);
    let mut faults = skein_lib::List::with_capacity(4);
    let defaults = crate::schema::declarations(&documents, &mut faults);
    assert_eq!((defaults.sound(b"core"), defaults.sound(b"motd")), (true, false));
}
#[test]
fn sound_tasks_are_looked_up_in_name_order() {
    let documents = fixtures::declarations(&[
        (b"a-b", Some(fixtures::declaration(&[], &[]))),
        (b"a", Some(fixtures::declaration(&[], &[]))),
    ]);
    let mut faults = skein_lib::List::with_capacity(4);
    let defaults = crate::schema::declarations(&documents, &mut faults);
    assert_eq!((defaults.sound(b"a"), defaults.sound(b"a-b")), (true, true));
}
