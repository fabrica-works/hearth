use crate::{
    diff::{Change, PartChange, Rebuild, TargetChange},
    parts::data::{Data, DataChange, DataClass, DataEntry, DataKeyed, DataValue},
    records::{Part, PartRecord, Record},
};
pub fn value(class: DataClass) -> DataValue {
    DataValue { class, path: b"/var/lib/iwd".as_slice().into() }
}
pub fn spec() -> Record {
    let mut spec = super::x1_prod();
    spec.parts = Box::new([Part {
        digest: [0; 32],
        record: PartRecord::Data(Data {
            entries: Box::new([
                DataEntry { name: b"iwd".as_slice().into(), value: value(DataClass::Core) },
                DataEntry {
                    name: b"journal".as_slice().into(),
                    value: DataValue { class: DataClass::Core, path: b"/var/log/journal".as_slice().into() },
                },
            ]),
        }),
    }]);
    Record::Spec(spec)
}
pub fn change() -> Record {
    Record::Change(Change {
        target: b"x1-prod".as_slice().into(),
        change: TargetChange::Changed {
            rebuild: Rebuild::SlotOnly,
            parts: Box::new([PartChange::Data(DataChange {
                entries: Box::new([DataKeyed::Changed {
                    key: b"iwd".as_slice().into(),
                    before: value(DataClass::Core),
                    after: value(DataClass::Data),
                }]),
            })]),
        },
    })
}
