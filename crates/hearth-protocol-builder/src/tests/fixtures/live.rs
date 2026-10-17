use crate::render::parts::live::{Action, Consumer, Covered, Live};
use skein_lib::bytes::copy_of;
#[must_use]
pub fn live_records() -> [Live; 5] {
    [Action::Reload, Action::Restart, Action::ReloadOrRestart, Action::Start, Action::None].map(|action| Live {
        paths: Box::new([Covered {
            path: copy_of(b"/etc/mailname"),
            consumers: Box::new([
                Consumer { task: copy_of(b"mail"), unit: Some(copy_of(b"postfix.service")), action },
                Consumer { task: copy_of(b"tool"), unit: None, action: Action::None },
            ]),
        }]),
    })
}

#[must_use]
pub fn live_specs() -> [crate::render::parts::Spec; 5] {
    live_records().map(|live| {
        let mut spec = super::spec();
        let mut parts = skein_lib::List::with_capacity(2);
        for part in spec.parts {
            parts.push(part).expect("config part");
        }
        parts
            .push(crate::render::parts::Part {
                digest: hearth_lib::encode::Digest([0; 32]),
                record: crate::render::parts::PartRecord::Live(live),
            })
            .expect("live part");
        spec.parts = parts.into_boxed();
        spec
    })
}
