use hearth_conformance_world::{
    fake_mmdebstrap::{self, Spawned},
    resolve,
};

#[test]
fn the_fake_prints_each_mmdebstrap_recording_as_recorded() {
    let mut mirror = Some(fake_mmdebstrap::committed());
    let mut stanzas_compared = 0;
    for (name, real) in hearth_recordings::all("mmdebstrap") {
        stanzas_compared += usize::from(real.stanzas.is_some());
        let case = resolve::case(&name);
        let fake =
            fake_mmdebstrap::run(&mut mirror, case, &Spawned { argv: &real.argv, env: &real.env, dir: &real.dir });
        assert_eq!(fake_mmdebstrap::differ(case, &real, &fake, real.stanzas.as_deref()), None, "{name}");
    }
    assert!(stanzas_compared >= 1, "test-min’s recorded stanzas were compared");
}
