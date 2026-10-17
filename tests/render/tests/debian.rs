use hearth_domain_render::{
    errors::{Kind, Schema},
    vocabulary::{Answer, Decoded},
};
use hearth_render_world::{fixtures, parent};
#[test]
fn a_target_without_debian_has_no_spec_and_one_error() {
    for seed in 0..8 {
        let mut repository = fixtures::x1_prod();
        for (_, decoded) in &mut repository.documents {
            if let Decoded::Document(document) = decoded {
                document.layer.debian = None;
            }
        }
        let run = parent::run(&repository, &[], seed);
        let Answer::Error(error) = fixtures::only(&run.answers) else { panic!("one schema error") };
        assert_eq!(error.kind, Kind::Schema(Schema::Missing { setting: b"debian".as_slice().into() }), "seed {seed}");
        assert_eq!(error.task, None, "seed {seed}");
    }
}
