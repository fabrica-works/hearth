use hearth_render_world::fixtures::laws as inputs;

#[test]
fn an_empty_patch_leaves_the_target_as_it_was() {
    for (seed, value, _) in inputs::cases() {
        let target = inputs::object(value);
        assert_eq!(inputs::merged(&[&target, &inputs::EMPTY]), inputs::merged(&[&target]), "seed {seed}");
    }
}
#[test]
fn merging_a_patch_twice_is_merging_it_once() {
    for (seed, target, patch) in inputs::cases() {
        inputs::idempotent(&target, &patch, seed);
    }
}
#[test]
fn a_scalar_or_list_patch_replaces_whole() {
    for (seed, target, _) in inputs::cases() {
        for patch in inputs::REPLACEMENTS {
            assert_eq!(inputs::merged(&[&target, patch]), Some(patch.clone()), "seed {seed}: {patch:?}");
        }
    }
}
#[test]
fn an_absent_key_keeps_its_value() {
    for (seed, value, patch) in inputs::cases() {
        let target = inputs::object(value);
        let patch = hearth_render_world::json::object(&[("other", patch)]);
        assert_eq!(inputs::merged(&[&target, &patch]), inputs::merged(&[&patch, &target]), "seed {seed}");
    }
}
#[test]
fn no_merge_result_holds_null() {
    for (seed, target, patch) in inputs::cases() {
        assert_eq!(inputs::merged(&[&target, &patch]).as_ref().map(inputs::has_null), Some(false), "seed {seed}");
    }
}
#[test]
fn merge_equals_format_md_4_1s_pseudocode() {
    for (seed, target, patch) in inputs::cases() {
        inputs::reference_agrees(&target, &patch, seed);
        inputs::reference_agrees(&target, &inputs::EMPTY, seed);
    }
}
#[test]
fn parts_with_different_records_have_different_digests() {
    let parts = inputs::parts();
    assert_eq!(
        parts.keys().copied().collect::<Vec<_>>(),
        ["boot", "config", "data", "exports", "live", "packages", "secrets", "software"]
    );
    for (name, parts) in parts {
        for left in &parts {
            for right in &parts {
                assert_eq!(left.record == right.record, left.digest == right.digest, "part {name}");
            }
        }
    }
}
