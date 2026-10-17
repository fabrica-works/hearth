use hearth_git_world::{
    fixtures, generate,
    world::{self, Outcome, Settings},
};
use hearth_protocol_git::listing::Error;
use skein_lib::{Rng, stream::Fault};
#[test]
fn a_generated_listing_decodes_to_its_entries_whatever_the_cuts() {
    for seed in 0..16 {
        let listing = generate::listing(&mut Rng::new(seed), &fixtures::SMALL);
        let outcome = world::list(&listing.stdout, &fixtures::SMALL, &Settings::cut(seed));
        assert_eq!(outcome, Outcome::Done(listing.entries), "seed {seed}");
    }
}
#[test]
fn a_close_in_every_state_answers_closed_and_demands_nothing() {
    let listing = generate::listing(&mut Rng::new(8), &fixtures::SMALL);
    for close in 0..8 {
        let settings = Settings { close: Some(close), ..Settings::cut(12) };
        let outcome = world::list(&listing.stdout, &fixtures::SMALL, &settings);
        match outcome {
            Outcome::Closed | Outcome::Done(_) => {}
            Outcome::Failed(error) => panic!("unexpected {error:?}"),
        }
    }
}
#[test]
fn a_stream_that_fails_mid_listing_fails_the_listing() {
    let listing = generate::listing(&mut Rng::new(8), &fixtures::SMALL);
    for end in 0..listing.stdout.len() {
        let settings = Settings { end: Some(end), fail: true, ..Settings::cut(12) };
        assert_eq!(
            world::list(&listing.stdout, &fixtures::SMALL, &settings),
            Outcome::Failed(Error::Stream(Fault::Reset)),
            "failed after {end}"
        );
    }
}
