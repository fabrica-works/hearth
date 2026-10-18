//! Shared roots and allocator checks for the memory stories.
use crate::expressions::Roots;
use skein_world::domain::heap::Meter;

pub fn each<T>(bound: u64, rows: impl IntoIterator<Item = (T, &'static str)>, run: impl Fn(&T, &Roots)) {
    let roots = super::wide_roots();
    for (input, what) in rows {
        peak(&Meter::new(), bound, &what, || run(&input, &roots));
    }
}
pub fn peak(meter: &Meter, bound: u64, what: &dyn core::fmt::Debug, run: impl FnOnce()) {
    meter.start();
    run();
    let measured = meter.end();
    meter.check(measured, bound, what);
}
