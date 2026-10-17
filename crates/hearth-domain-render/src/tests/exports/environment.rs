use crate::{
    errors::{Cause, Kind, Schema, TypeName},
    evaluate::{FaultKind, Value},
    exports::Cache,
    limits::SHIPPED,
    tests::fixtures::{
        self,
        export_readers::{SMALL_EXPORTED, keys, readers},
        only,
    },
};
use skein_lib::bytes::copy_of;
#[test]
fn a_reader_gets_every_target_of_its_environment_that_has_the_task() {
    let fleet = readers(false);
    let cache = fleet.cache(b"carbon-prod");
    let Value::Map(reads) = fleet.root(&cache, b"carbon-prod").expect("root") else { panic!("map") };
    let Some(Value::Map(targets)) = &only(&reads).value else { panic!("targets") };
    assert_eq!(keys(targets).as_ref(), [b"ion-prod"]);
}
#[test]
fn the_reader_itself_is_included_when_it_has_the_task() {
    let fleet = readers(true);
    let cache = fleet.cache(b"carbon-prod");
    let Value::Map(reads) = fleet.root(&cache, b"carbon-prod").expect("root") else { panic!("map") };
    let Some(Value::Map(targets)) = &only(&reads).value else { panic!("targets") };
    assert_eq!(keys(targets).as_ref(), [b"carbon-prod".as_slice(), b"ion-prod"]);
}
#[test]
fn a_task_nobody_has_gives_an_empty_exports_map() {
    let mut fleet = readers(false);
    fleet.remove_exporters();
    let cache = Cache::new(&SHIPPED);
    let Value::Map(reads) = fleet.root(&cache, b"carbon-prod").expect("empty root") else { panic!("map") };
    assert_eq!(only(&reads).value, Some(Value::Map(Box::new([]))));
}
#[test]
fn an_exporters_settings_fault_fails_the_reader_naming_the_exporter() {
    let mut fleet = readers(false);
    fleet.setting_fault();
    let cache = fleet.cache(b"carbon-prod");
    assert_eq!(
        fleet.root(&cache, b"carbon-prod").expect_err("bad settings").kind,
        Kind::Exported(crate::errors::Exported {
            exporter: copy_of(b"ion-prod"),
            task: copy_of(b"vpn"),
            cause: Cause::Schema(Schema::Wrong { expected: TypeName::Int })
        })
    );
}
#[test]
fn an_exporter_with_a_broken_document_fails_the_reader() {
    let mut fleet = readers(false);
    fleet.refused(b"hosts/ion/ion-prod.json");
    let cache = fleet.cache(b"carbon-prod");
    assert_eq!(
        fleet.root(&cache, b"carbon-prod").expect_err("broken document").at,
        Some(crate::errors::Place {
            path: copy_of(b"hosts/ion/ion-prod.json"),
            position: Some(fixtures::composition::at(2, 3))
        })
    );
}
#[test]
fn a_refused_hosts_document_fails_the_reader_with_its_fault() {
    let mut fleet = readers(false);
    fleet.refused(b"hosts/unknown/unknown-prod.json");
    let cache = Cache::new(&SHIPPED);
    assert_eq!(
        fleet.root(&cache, b"carbon-prod").expect_err("hidden target").kind,
        Kind::Exported(crate::errors::Exported {
            exporter: copy_of(b"unknown"),
            task: copy_of(b"vpn"),
            cause: Cause::Decode(crate::errors::Decode::Unexpected)
        })
    );
}
#[test]
fn an_exporters_other_faults_leave_its_readers_rendering() {
    let mut fleet = readers(false);
    fleet.faults.push(fixtures::error_at(b"tasks/unrelated/tree/etc/a.j2", 1, 1, b"ion-prod"));
    let cache = fleet.cache(b"carbon-prod");
    let Value::Map(reads) = fleet.root(&cache, b"carbon-prod").expect("unrelated fault") else { panic!("map") };
    assert_eq!(keys(&reads).as_ref(), [b"vpn"]);
}
#[test]
fn a_pair_is_evaluated_once_for_all_its_readers() {
    let fleet = readers(false);
    let mut cache = fleet.cache(b"carbon-prod");
    assert_eq!(fleet.prime(&mut cache, b"ion-prod"), 0);
}
#[test]
fn exports_held_past_their_limit_fail_the_reader_with_an_evaluation_limit() {
    let mut fleet = readers(false);
    fleet.limits = SMALL_EXPORTED;
    let cache = fleet.cache(b"carbon-prod");
    assert_eq!(
        fleet.root(&cache, b"carbon-prod").expect_err("retained limit").kind,
        Kind::Evaluation(FaultKind::Limit)
    );
}
