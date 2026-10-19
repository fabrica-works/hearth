use hearth_accounts_world::{
    fixtures::{self, FILES, LIMITS},
    world,
};
use hearth_protocol_accounts::{read, worst_case};
use skein_heap::Counting;
#[global_allocator]
static HEAP: Counting = Counting;
#[test]
fn a_read_at_its_limits_holds_no_more_than_its_worst_case() {
    for file in FILES {
        for bytes in fixtures::at_the_limits(file, &LIMITS) {
            let lines = read(file, &bytes, &LIMITS).unwrap();
            assert!(lines.len() > 1_000, "{file:?}: the fixture fills the limits");
            drop(lines);
            let most = world::read_metered(file, &bytes, &LIMITS);
            assert!(most <= worst_case(&LIMITS).unwrap(), "{file:?}");
        }
    }
}
