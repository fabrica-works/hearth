pub fn live_spec() -> crate::records::Record {
    let crate::records::Record::Spec(mut spec) = super::secret_spec() else { panic!("spec") };
    spec.parts = Box::new([crate::records::Part {
        digest: [0; 32],
        record: crate::records::PartRecord::Live(crate::parts::live::Live {
            paths: Box::new([crate::parts::live::Covered {
                path: Box::from(&b"/opt/tool/"[..]),
                consumers: Box::new([crate::parts::live::Consumer {
                    task: Box::from(&b"motd"[..]),
                    unit: None,
                    action: crate::parts::live::Action::None,
                }]),
            }]),
        }),
    }]);
    crate::records::Record::Spec(spec)
}
