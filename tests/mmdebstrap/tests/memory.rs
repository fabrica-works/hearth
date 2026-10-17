use hearth_mmdebstrap_world::{fixtures, world};
use hearth_protocol_mmdebstrap::{limits, output};
use skein_heap::Counting;
#[global_allocator]
static HEAP: Counting = Counting;
#[test]
fn an_output_reader_at_its_limits_holds_no_more_than_its_worst_case() {
    let bytes = fixtures::at_the_limits(&limits::SHIPPED);
    let most = world::read_output_metered(&bytes, &limits::SHIPPED);
    assert!(most <= output::worst_case(&limits::SHIPPED).unwrap());
}
