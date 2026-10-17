pub fn software_spec() -> crate::records::Record {
    use crate::parts::software::{Download, Fetch, How, Software};
    let crate::records::Record::Spec(mut spec) = super::secret_spec() else { panic!("spec") };
    spec.parts = Box::new([crate::records::Part {
        digest: [0; 32],
        record: crate::records::PartRecord::Software(Software {
            downloads: Box::new([
                Download {
                    path: b"/opt/a".as_slice().into(),
                    sha256: [1; 32],
                    owner: b"root".as_slice().into(),
                    group: b"staff".as_slice().into(),
                    how: How::File { mode: 0o755 },
                },
                Download {
                    path: b"/opt/b".as_slice().into(),
                    sha256: [1; 32],
                    owner: b"root".as_slice().into(),
                    group: b"staff".as_slice().into(),
                    how: How::Tar { strip: 1 },
                },
            ]),
            scripts: Box::new([super::directives::script([1; 32])]),
            fetch: Box::new([Fetch { sha256: [1; 32], url: b"https://example.org/a".as_slice().into() }]),
        }),
    }]);
    crate::records::Record::Spec(spec)
}
