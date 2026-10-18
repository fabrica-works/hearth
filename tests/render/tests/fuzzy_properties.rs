use hearth_render_world::fixtures::laws;
use hearth_render_world::fixtures::properties as inputs;
#[test]
fn generated_patches_obey_the_laws_and_the_reference() {
    for seed in laws::ROUNDS {
        let (target, patch) = laws::patches(seed, laws::WIDE_DEPTH);
        laws::reference_agrees(&target, &patch, seed);
        laws::idempotent(&target, &patch, seed);
        let object = laws::object(target);
        assert_eq!(laws::merged(&[&object, &laws::EMPTY]), laws::merged(&[&object]), "seed {seed}");
        assert_eq!(laws::merged(&[&object, &patch]).as_ref().map(laws::has_null), Some(false), "seed {seed}");
    }
}
#[test]
fn generated_repositories_and_edits_keep_every_property_0() {
    inputs::sweep(0);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_1() {
    inputs::sweep(1);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_2() {
    inputs::sweep(2);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_3() {
    inputs::sweep(3);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_4() {
    inputs::sweep(4);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_5() {
    inputs::sweep(5);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_6() {
    inputs::sweep(6);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_7() {
    inputs::sweep(7);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_8() {
    inputs::sweep(8);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_9() {
    inputs::sweep(9);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_10() {
    inputs::sweep(10);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_11() {
    inputs::sweep(11);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_12() {
    inputs::sweep(12);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_13() {
    inputs::sweep(13);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_14() {
    inputs::sweep(14);
}
