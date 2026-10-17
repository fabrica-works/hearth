use hearth_git_world::{
    fixtures, generate,
    world::{self, Outcome, Settings},
};
use hearth_protocol_git::listing::Error;
use skein_lib::{Rng, stream::Fault};
#[test]
fn random_listings_cut_ended_failed_and_closed_settle() {
    for seed in 0..512 {
        let mut rng = Rng::new(seed);
        let listing = generate::listing(&mut rng, &fixtures::SMALL);
        let end = usize::try_from(rng.between(0, listing.stdout.len() as u64)).expect("small input");
        let settings = Settings {
            end: Some(end),
            fail: rng.chance(250),
            close: if rng.chance(250) { Some(u32::try_from(rng.below(8)).expect("small call")) } else { None },
            ..Settings::cut(seed)
        };
        let outcome = world::list(&listing.stdout, &fixtures::SMALL, &settings);
        let expected = if settings.fail {
            Outcome::Failed(Error::Stream(Fault::Reset))
        } else if end == 0 || listing.stdout[end - 1] == 0 {
            Outcome::Done(
                listing
                    .entries
                    .iter()
                    .take(
                        usize::try_from(skein_lib::bytes::count(&listing.stdout[..end], b"\0", u32::MAX))
                            .expect("small count"),
                    )
                    .cloned()
                    .collect(),
            )
        } else {
            Outcome::Failed(Error::Truncated)
        };
        assert!(outcome == Outcome::Closed || outcome == expected, "seed {seed}: {outcome:?}, expected {expected:?}");
    }
}

#[test]
fn random_batches_cut_ended_failed_and_closed_settle() {
    use hearth_git_world::world::BatchOutcome;
    use hearth_protocol_git::batch;
    for seed in 0..512 {
        let mut rng = Rng::new(seed);
        let batch = generate::batch(&mut rng, &fixtures::SMALL);
        let end = usize::try_from(rng.between(0, batch.stdout.len() as u64)).expect("small input");
        let settings = Settings {
            end: Some(end),
            fail: rng.chance(250),
            close: if rng.chance(250) { Some(u32::try_from(rng.below(24)).expect("small call")) } else { None },
            ..Settings::cut(seed)
        };
        let expected = if end == batch.stdout.len() {
            BatchOutcome::Done(batch.blobs)
        } else if settings.fail {
            BatchOutcome::Failed(batch::Error::Stream(Fault::Reset))
        } else {
            BatchOutcome::Failed(batch::Error::Truncated)
        };
        let outcome = world::read(&batch.stdout, &batch.requests, &fixtures::SMALL, &settings);
        assert!(
            outcome == BatchOutcome::Closed || outcome == expected,
            "seed {seed}: {outcome:?}, expected {expected:?}"
        );
    }
}
