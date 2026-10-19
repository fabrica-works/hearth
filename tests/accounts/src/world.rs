use hearth_protocol_accounts::{File, limits::Limits, read, worst_case};
use skein_heap::Meter;
/// The most heap one read holds, checked against its worst case; the result is dropped after.
#[must_use]
pub fn read_metered(file: File, bytes: &[u8], limits: &Limits) -> u64 {
    let bound = worst_case(limits).expect("bounded fixture value");
    let meter = Meter::new();
    meter.start();
    let result = read(file, bytes, limits);
    let step = meter.end();
    drop(result);
    meter.check(step, bound, &file)
}
