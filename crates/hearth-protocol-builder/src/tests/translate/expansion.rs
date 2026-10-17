use crate::{records, render, tests::fixtures, translate};
#[test]
fn each_entry_and_condition_translates_to_render_and_back() {
    for (row, (entry, input)) in fixtures::every_entry_condition().into_iter().enumerate() {
        let entry = translate::declarations::entry(entry, &crate::limits::SHIPPED.template).expect("entry compiles");
        let back = super::back::expression(entry.when.expect("condition").expression);
        assert_eq!(back, input, "row {row}: condition");
    }
}
#[test]
fn each_entry_for_each_translates_to_render_and_back() {
    for (row, (entry, input)) in fixtures::every_entry_condition().into_iter().enumerate() {
        let entry = translate::declarations::entry(entry, &crate::limits::SHIPPED.template).expect("entry compiles");
        let back = super::back::expression(entry.for_each.expect("iterable").expression);
        assert_eq!(back, input, "row {row}: iterable");
    }
}
#[test]
fn a_declarations_maps_entries_and_plain_keys_keep_their_positions() {
    let declaration = fixtures::read_declaration(br#"{"groups":{"ops":{}},"users":{"ann":{}}}"#);
    let [groups, users] = declaration.entries.as_ref() else { panic!("two maps") };
    assert_eq!(
        (groups.map, users.map),
        (render::revision::declaration::Map::Groups, render::revision::declaration::Map::Users)
    );
    assert_eq!(
        fixtures::only(&groups.entries).key,
        render::revision::declaration::Interpolated::Plain {
            at: render::revision::Position { line: 1, column: 12 },
            text: skein_lib::bytes::copy_of(b"ops")
        }
    );
}
#[test]
fn an_entrys_interpolated_key_reaches_render_compiled() {
    let declaration = fixtures::read_declaration(br#"{"groups":{"{{ value }}":{"for_each":"settings.tags"}}}"#);
    let key = &fixtures::only(&fixtures::only(&declaration.entries).entries).key;
    let render::revision::declaration::Interpolated::Parts { at, parts } = key else { panic!("compiled parts") };
    assert_eq!(*at, render::revision::Position { line: 1, column: 12 });
    let render::evaluate::template::Does::Output(expression) = &fixtures::only(parts).does else { panic!("output") };
    assert_eq!(
        expression.operations.as_ref(),
        &[render::evaluate::Operation {
            at: 3,
            op: render::evaluate::Op::Name {
                name: skein_lib::bytes::copy_of(b"value"),
                missing: render::evaluate::Missing::Error
            }
        }]
    );
}
fn collision_back(collision: records::errors::Collision) -> render::errors::Collision {
    match collision {
        records::errors::Collision::Key { key } => render::errors::Collision::Key { key },
    }
}
#[test]
fn each_collision_translates_to_records_and_back() {
    for (row, input) in fixtures::collision_faults().into_iter().enumerate() {
        let records::errors::Kind::Collision(collision) = translate::errors::error(render::errors::Error {
            kind: render::errors::Kind::Collision(input.clone()),
            ..fixtures::decode_error()
        })
        .kind
        else {
            panic!("collision")
        };
        let back = collision_back(collision);
        assert_eq!(back, input, "row {row}: collision");
    }
}
