use hearth_builder_world::{
    domain::{Cut, Failing},
    fixtures,
};
use hearth_domain_builder::{
    build::vocabulary::Why,
    vocabulary::{Answer, Request},
};
use hearth_domain_render::{
    keys::Resolution,
    vocabulary::{Outcome, Request as RenderRequest},
};
use hearth_domain_store::vocabulary::{Helper, Request as StoreRequest};
use hearth_protocol_tree::layer::Inputs;

#[test]
fn a_cold_restart_settles_what_a_cut_left_behind() {
    fixtures::each_domain(|mut run, seed| {
        run.cut_at(Cut::BeforeKeep);
        let left = run.finish();
        assert!(left.strays() > 0, "seed {seed}: a synced arrival awaits its rename");
        let after = left.restart().finish();
        assert_eq!(after.strays(), 0, "seed {seed}: domain/store.md, 11");
        assert_eq!(after.outcome, Some(Outcome::Done), "seed {seed}: call answered");
        assert_eq!(after.answers.len(), 1, "seed {seed}: one spec");
        assert!(matches!(fixtures::only(&after.answers), Answer::Spec(_)), "seed {seed}");
    });
}

#[test]
fn a_thin_repository_renders_through_the_three_domains() {
    fixtures::each_settled(&fixtures::thin(), |run, seed| {
        let digest = fixtures::hex_digest(fixtures::spec(&run).digest.0);
        assert_eq!(
            digest.chars().take(12).collect::<String>(),
            "23656fdf46b7",
            "seed {seed}: fixtures::THIN_TEXT's spec line"
        );
        assert_eq!(run.outcome, Some(Outcome::Done), "seed {seed}");
    });
}

#[test]
fn a_busy_lock_is_asked_again_every_200_ms_until_released() {
    fixtures::each_domain(|mut run, seed| {
        run.hold_lock_until(fixtures::LOCK_RELEASE);
        let run = run.finish();
        assert!(run.lock_times.len() >= 2, "seed {seed}: busy lock retried");
        assert!(
            run.lock_times.windows(2).all(|times| {
                let [before, after] = times else { unreachable!("two times") };
                let span = after.saturating_since(*before);
                span >= fixtures::LOCK_RETRY
                    && span <= fixtures::LOCK_RETRY.saturating_add(skein_lib::Duration::from_millis(20))
            }),
            "seed {seed}: retries follow each Busy by 200 ms plus its latency"
        );
        assert!(run.now() >= fixtures::LOCK_RELEASE, "seed {seed}: it waited for release");
        assert_eq!(run.outcome, Some(Outcome::Done), "seed {seed}");
    });
}

#[test]
fn a_file_entering_twice_is_renamed_once() {
    let files = fixtures::thin_with(&[fixtures::DUPLICATE_BLOB]);
    let scenario = fixtures::Scenario { files: &files, ..fixtures::thin() };
    fixtures::each_settled(&scenario, |run, seed| {
        assert_eq!(
            run.shared
                .borrow()
                .renames
                .iter()
                .filter(|rename| rename.to.as_ref() == fixtures::THIN_MOTD_SHA256_HEX.as_bytes())
                .count(),
            1,
            "seed {seed}: the file referenced twice is renamed once"
        );
    });
}

#[test]
fn a_resolved_key_is_recorded_and_the_next_call_finds_it_known() {
    fixtures::each_scenario(&fixtures::resolving(fixtures::THIN), |run, seed| {
        let resolved = run.finish();
        assert_eq!(resolved.outcome, Some(Outcome::Done), "seed {seed}: resolved");
        let record = fixtures::recorded(&resolved);
        let known = resolved.restart().finish();
        assert_eq!(fixtures::recorded(&known), record, "seed {seed}: cold call loads the same record");
        assert!(
            !known.requests().iter().any(|request| matches!(request, Request::Render(RenderRequest::Resolve { .. }))),
            "seed {seed}: no second resolution"
        );
        assert!(
            matches!(fixtures::spec(&known).resolution.as_deref(), Some(Resolution::Known { .. })),
            "seed {seed}: known closure"
        );
    });
}

#[test]
fn every_request_gets_one_terminal_whatever_the_latencies() {
    fixtures::each_settled(&fixtures::thin(), |run, seed| {
        assert!(run.referee.asked > 0, "seed {seed}: requests were made");
        assert_eq!(run.referee.asked, run.referee.ended, "seed {seed}: Ledger closes every obligation");
        assert_eq!(
            run.requests().iter().filter(|request| matches!(request, Request::Store(StoreRequest::Lock))).count(),
            1,
            "seed {seed}: one builder holds the store"
        );
    });
}

#[test]
fn resolution_keys_equal_a_naive_transitive_closure_of_the_fake_universe() {
    let files = fixtures::thin_with(&[fixtures::WITH_DEPENDENCIES]);
    fixtures::each_scenario(&fixtures::resolving(&files), |run, seed| {
        let run = fixtures::first_alternatives(run).finish();
        let record = fixtures::recorded(&run);
        let universe = run.shared.borrow();
        let packages = naive::closure(&universe.universe, b"pkg-23");
        assert_eq!(
            record.closure.iter().map(|package| package.name.as_ref()).collect::<Vec<_>>(),
            packages.iter().map(|package| package.name.as_ref()).collect::<Vec<_>>(),
            "seed {seed}: the independent closure"
        );
        let digest = naive::digest(&packages);
        assert_eq!(record.digest, digest, "seed {seed}: the closure's key");
        let spec = fixtures::spec(&run);
        let Some(Resolution::Known { keys, .. }) = spec.resolution.as_deref() else { unreachable!("resolved keys") };
        assert_eq!(*keys, naive::layers(digest, spec), "seed {seed}: dependent layer keys");
    });
}

#[path = "domain/naive.rs"]
mod naive;

/// Render's keys and config part for THIN's target, resolved in the same world.
fn rendered_keys(seed: u64) -> (hearth_domain_render::keys::Keys, hearth_lib::encode::Digest) {
    let run = hearth_builder_world::domain::Run::new(&fixtures::resolving(fixtures::THIN), seed).finish();
    let spec = fixtures::spec(&run);
    let Some(Resolution::Known { keys, .. }) = spec.resolution.as_deref() else { unreachable!("resolved keys") };
    (keys.clone(), hearth_domain_render::keys::part(spec, b"config"))
}
/// The layers of a build from an empty store: their answers, with the state each is in.
fn laid_out(keys: &hearth_domain_render::keys::Keys, found: hearth_domain_builder::vocabulary::State) -> Vec<Answer> {
    use hearth_domain_builder::{build::Layer, vocabulary::State};
    [
        (Layer::Packages, keys.packages, found),
        (Layer::Software, keys.software, found),
        (Layer::Config, keys.config, if found == State::Found { State::Found } else { State::ToBuild }),
    ]
    .into_iter()
    .map(|(layer, key, state)| Answer::Layer { layer, key, state })
    .collect()
}
fn runs(run: &hearth_builder_world::domain::Run) -> usize {
    run.requests().iter().filter(|request| matches!(request, Request::Run { .. })).count()
}
fn helped(run: &hearth_builder_world::domain::Run) -> Vec<&Helper> {
    run.requests()
        .iter()
        .filter_map(
            |request| if let Request::Store(StoreRequest::Helper(helper)) = request { Some(helper) } else { None },
        )
        .collect()
}
#[test]
fn a_build_from_an_empty_store_publishes_the_empty_packages_software_and_config_layers() {
    use hearth_domain_builder::vocabulary::State;
    use hearth_domain_store::layers;
    fixtures::each_settled(&fixtures::build::building(), |run, seed| {
        let (keys, part) = rendered_keys(seed);
        assert_eq!(run.answers, laid_out(&keys, State::ToBuild), "seed {seed}: three layers to build, none failed");
        assert_eq!(run.outcome, Some(Outcome::Done), "seed {seed}: render's own outcome");
        let empty = layers::empty();
        let chain = [
            (empty, None),
            (keys.packages, Some(empty)),
            (keys.software, Some(keys.packages)),
            (keys.config, Some(keys.software)),
        ];
        let mut uuids = Vec::new();
        for (key, parent) in chain {
            let record = fixtures::build::record(&run.shared, key).expect("a published layer has its record");
            assert_eq!(record.key, key, "seed {seed}");
            assert_eq!(record.parent, parent, "seed {seed}: the layer beneath is the parent");
            assert_eq!(
                layers::key(&hearth_protocol_builder::translate::store::layer(record.clone())),
                Some(key),
                "seed {seed}: domain/store.md, 6.2: the key recomputes from kind, parent and inputs"
            );
            if let Some(parent) = parent {
                let beneath = fixtures::build::record(&run.shared, parent).expect("the parent's record");
                assert_eq!(record.subvolume.parent_uuid, Some(beneath.subvolume.uuid), "seed {seed}: lineage (4.5)");
            }
            assert!(!uuids.contains(&record.subvolume.uuid), "seed {seed}: one subvolume each");
            uuids.push(record.subvolume.uuid);
            let hex = fixtures::hex_digest(key.0);
            for published in [hex.clone(), format!("{hex}.tree")] {
                assert!(fixtures::build::laid(&run.shared, &published), "seed {seed}: {published}");
            }
            for gone in [format!("{hex}.staging"), format!("{hex}.tree.tmp")] {
                assert!(!fixtures::build::laid(&run.shared, &gone), "seed {seed}: {gone} is renamed away");
            }
        }
        let config = fixtures::build::record(&run.shared, keys.config).expect("config record");
        assert_eq!(config.inputs, Inputs::Config { part }, "seed {seed}: the part the key covers");
        assert_eq!(runs(&run), 11, "seed {seed}: three, three and five steps");
        assert_eq!(run.strays(), 0, "seed {seed}: no staging and no temporary remain");
        assert_eq!(run.referee.asked, run.referee.ended, "seed {seed}: every request has its terminal");
    });
}
#[test]
fn a_second_build_of_the_same_spec_stages_and_runs_nothing() {
    use hearth_domain_builder::vocabulary::State;
    fixtures::each_settled(&fixtures::build::building(), |run, seed| {
        let (keys, _) = rendered_keys(seed);
        let again = run.restart().finish();
        assert_eq!(again.answers, laid_out(&keys, State::Found), "seed {seed}: all three published");
        assert!(runs(&again) == 0 && helped(&again).is_empty(), "seed {seed}: no Run and no Stage");
        assert!(
            !again.requests().iter().any(|request| matches!(request, Request::Store(StoreRequest::Create { .. }))),
            "seed {seed}: the empty layer is published"
        );
        assert_eq!(again.outcome, Some(Outcome::Done), "seed {seed}");
    });
}
#[test]
fn a_changed_config_part_builds_config_alone_staged_from_the_published_software_layer() {
    use hearth_domain_builder::{build::Layer, vocabulary::State};
    fixtures::each_settled(&fixtures::build::building(), |run, seed| {
        let (keys, old) = rendered_keys(seed);
        fixtures::build::edit_config(&run.shared);
        let rebuilt = run.restart().finish();
        let [packages, software, Answer::Layer { layer: Layer::Config, key: config, state: State::ToBuild }] =
            rebuilt.answers.as_slice()
        else {
            panic!("seed {seed}: packages and software found, config to build: {:?}", rebuilt.answers);
        };
        assert_eq!(
            [packages, software],
            [
                &Answer::Layer { layer: Layer::Packages, key: keys.packages, state: State::Found },
                &Answer::Layer { layer: Layer::Software, key: keys.software, state: State::Found }
            ],
            "seed {seed}: only the config key moves"
        );
        assert_ne!(*config, keys.config, "seed {seed}: a new part, a new key");
        assert_eq!(
            helped(&rebuilt),
            [
                &Helper::Snapshot { parent: keys.software, staging: *config },
                &Helper::Walk { layer: *config, parent: Some(keys.software) },
                &Helper::Freeze { layer: *config },
            ],
            "seed {seed}: staged from the published software layer, then sealed"
        );
        assert_eq!(runs(&rebuilt), 5, "seed {seed}: the config layer's five steps, nothing beneath");
        let record = fixtures::build::record(&rebuilt.shared, *config).expect("the new layer's record");
        assert_eq!(record.parent, Some(keys.software), "seed {seed}");
        assert_ne!(record.inputs, Inputs::Config { part: old }, "seed {seed}: it names the new part");
        assert!(fixtures::build::record(&rebuilt.shared, keys.config).is_some(), "seed {seed}: the old layer stays");
        assert_eq!(rebuilt.strays(), 0, "seed {seed}");
    });
}
#[test]
fn a_build_whose_config_layer_is_published_finds_all_three() {
    use hearth_domain_builder::{build::Layer, vocabulary::State};
    fixtures::each_scenario(&fixtures::build::building(), |run, seed| {
        let (keys, part) = rendered_keys(seed);
        fixtures::build::plant(&run.shared, &fixtures::build::config_layer(keys.config, keys.software, part));
        let found = run.finish();
        assert_eq!(
            found.answers,
            [
                Answer::Layer { layer: Layer::Packages, key: keys.packages, state: State::Found },
                Answer::Layer { layer: Layer::Software, key: keys.software, state: State::Found },
                Answer::Layer { layer: Layer::Config, key: keys.config, state: State::Found },
            ],
            "seed {seed}: domain/build.md, 4.1: a published config key is the output"
        );
        assert!(
            !found.requests().iter().any(|request| matches!(request, Request::Run { .. })),
            "seed {seed}: nothing runs"
        );
        assert_eq!(found.outcome, Some(Outcome::Done), "seed {seed}");
    });
}

/// Every step and helper request of a clean build, as a build can fail it: with the layer it is
/// in and what its failure is reported as (domain/build.md, 4.3).
fn failures(clean: &hearth_builder_world::domain::Run) -> Vec<(Failing, hearth_lib::encode::Digest, Why)> {
    let empty = hearth_domain_store::layers::empty();
    let (mut steps, mut helpers, mut layer) = (0, 0, empty);
    let mut points = Vec::new();
    for request in clean.requests() {
        match request {
            Request::Run { .. } => {
                points.push((Failing::Step(steps), layer, Why::Failed));
                steps += 1;
            }
            Request::Store(StoreRequest::Helper(helper)) => {
                let (key, why) = match helper {
                    Helper::Snapshot { staging, .. } => {
                        layer = *staging;
                        (*staging, Why::Stage)
                    }
                    Helper::Hash { layer: key } => (*key, Why::Stage),
                    Helper::Walk { layer: key, .. } | Helper::Freeze { layer: key } => {
                        (*key, if *key == empty { Why::Stage } else { Why::Seal })
                    }
                    Helper::Thaw { .. } | Helper::Apply { .. } | Helper::Empty { .. } | Helper::Delete { .. } => {
                        unreachable!("a clean build sends no {helper:?}")
                    }
                };
                points.push((Failing::Helper(helpers), key, why));
                helpers += 1;
            }
            Request::Progress(_)
            | Request::Render(_)
            | Request::Store(_)
            | Request::Unopened { .. }
            | Request::Answer(_)
            | Request::Reply { .. } => {}
        }
    }
    points
}
/// A build that fails at `failing` publishes nothing of the layer it is making, keeps the layers
/// beneath, leaves no staging layer and no temporary, and is built again by the next call.
fn fails_cleanly(failing: Failing, layer: hearth_lib::encode::Digest, why: Why, seed: u64) {
    use hearth_builder_world::domain::Run;
    let (keys, _) = rendered_keys(seed);
    let empty = hearth_domain_store::layers::empty();
    let chain = [empty, keys.packages, keys.software, keys.config];
    let at = chain.iter().position(|key| *key == layer).expect("a layer of the chain");
    let mut run = Run::new(&fixtures::build::building(), seed);
    run.fail(failing);
    let run = run.finish();
    let note = format!("seed {seed}: {failing:?} in layer {at}");
    assert!(
        matches!(run.answers.last(), Some(Answer::Failed { why: found, .. }) if *found == why),
        "{note}: {:?}",
        run.answers
    );
    assert_eq!(run.strays(), 0, "{note}: no staging layer and no temporary remain");
    for (index, key) in chain.iter().enumerate() {
        let hex = fixtures::hex_digest(key.0);
        if index < at {
            assert!(fixtures::build::record(&run.shared, *key).is_some(), "{note}: the layer beneath {index} stays");
            assert!(fixtures::build::laid(&run.shared, &hex), "{note}: and its subvolume");
            assert!(fixtures::build::laid(&run.shared, &format!("{hex}.tree")), "{note}: and its tree");
        } else {
            for name in [
                hex.clone(),
                format!("{hex}.record"),
                format!("{hex}.tree"),
                format!("{hex}.staging"),
                format!("{hex}.tree.tmp"),
            ] {
                assert!(!fixtures::build::laid(&run.shared, &name), "{note}: {name} is not left");
            }
        }
    }
    assert_eq!(run.referee.asked, run.referee.ended, "{note}: every request has its terminal");
    let again = run.restart().finish();
    assert_eq!(again.outcome, Some(Outcome::Done), "{note}: built again");
    assert!(!again.answers.iter().any(|answer| matches!(answer, Answer::Failed { .. })), "{note}: {:?}", again.answers);
    for key in chain {
        assert!(fixtures::build::record(&again.shared, key).is_some(), "{note}: the rebuild publishes layer {key:?}");
    }
    assert_eq!(again.strays(), 0, "{note}");
}
/// Fails each request of the wanted kind once, in the world of a seed drawn per request.
fn each_failure(wanted: impl Fn(Failing) -> bool) {
    use hearth_builder_world::domain::Run;
    let clean = Run::new(&fixtures::build::building(), 0).finish();
    let points: Vec<_> = failures(&clean).into_iter().filter(|(failing, _, _)| wanted(*failing)).collect();
    assert!(!points.is_empty(), "a build has requests to fail");
    for (index, (failing, layer, why)) in points.into_iter().enumerate() {
        let seed = u64::try_from(index).expect("few requests") % fixtures::WORLD_SEEDS.end;
        fails_cleanly(failing, layer, why, seed);
    }
}
#[test]
fn a_step_that_fails_in_any_layer_leaves_no_staging_and_no_temporary_and_keeps_the_layers_beneath() {
    each_failure(|failing| matches!(failing, Failing::Step(_)));
}
#[test]
fn a_helper_request_that_fails_leaves_no_staging_and_no_temporary_and_keeps_the_layers_beneath() {
    each_failure(|failing| matches!(failing, Failing::Helper(_)));
}
