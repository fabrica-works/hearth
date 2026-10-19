use hearth_builder_world::{domain::Cut, fixtures};
use hearth_domain_builder::vocabulary::{Answer, Request};
use hearth_domain_render::{
    keys::Resolution,
    vocabulary::{Outcome, Request as RenderRequest},
};
use hearth_domain_store::vocabulary::Request as StoreRequest;

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
#[test]
fn a_build_with_no_layers_plans_all_three_and_its_fetch_is_unavailable() {
    use hearth_domain_builder::{
        build::{Layer, Step, vocabulary::Why},
        vocabulary::State,
    };
    fixtures::each_settled(&fixtures::build::building(), |run, seed| {
        let (keys, _) = rendered_keys(seed);
        assert_eq!(
            run.answers,
            [
                Answer::Layer { layer: Layer::Packages, key: keys.packages, state: State::ToBuild },
                Answer::Layer { layer: Layer::Software, key: keys.software, state: State::ToBuild },
                Answer::Layer { layer: Layer::Config, key: keys.config, state: State::ToBuild },
                Answer::Failed { step: Step::Fetch(Layer::Packages), why: Why::Unavailable },
            ],
            "seed {seed}: render's keys, each to build, then the unavailable fetch"
        );
        assert_eq!(run.outcome, Some(Outcome::Done), "seed {seed}: render's own outcome");
        assert_eq!(run.referee.asked, run.referee.ended, "seed {seed}: the step has its terminal");
    });
}
#[test]
fn a_build_whose_config_layer_is_published_finds_all_three() {
    use hearth_domain_builder::{build::Layer, vocabulary::State};
    fixtures::each_settled(&fixtures::build::building(), |run, seed| {
        let (keys, part) = rendered_keys(seed);
        fixtures::build::plant(&run.shared, &fixtures::build::config_layer(keys.config, keys.software, part));
        let found = run.restart().finish();
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
