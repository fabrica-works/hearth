use hearth_git_world::{
    fixtures, generate,
    world::{self, BatchOutcome, Settings},
};
use skein_lib::Rng;
#[test]
fn generated_blobs_arrive_whole_in_order_whatever_the_cuts() {
    let mut empty = false;
    let mut missing = false;
    let mut nul = false;
    for seed in 0..32 {
        let batch = generate::batch(&mut Rng::new(seed), &fixtures::SMALL);
        for blob in &batch.blobs {
            match &blob.bytes {
                Some(bytes) => {
                    empty |= bytes.is_empty();
                    nul |= bytes.contains(&0);
                }
                None => missing = true,
            }
        }
        assert_eq!(
            world::read(&batch.stdout, &batch.requests, &fixtures::SMALL, &Settings::cut(seed)),
            BatchOutcome::Done(batch.blobs),
            "seed {seed}"
        );
    }
    assert!(empty && missing && nul, "drawn empty, absent and NUL-containing bodies");
}
#[test]
fn a_close_between_pieces_answers_closed_and_withdraws_its_demand() {
    let batch = generate::batch(&mut Rng::new(5), &fixtures::SMALL);
    for close in 0..32 {
        let settings = Settings { close: Some(close), ..Settings::cut(7) };
        let outcome = world::read(&batch.stdout, &batch.requests, &fixtures::SMALL, &settings);
        match outcome {
            BatchOutcome::Done(blobs) => assert_eq!(blobs, batch.blobs, "whole before close"),
            BatchOutcome::Closed => {}
            BatchOutcome::Failed(error) => panic!("unexpected {error:?}"),
        }
    }
}
