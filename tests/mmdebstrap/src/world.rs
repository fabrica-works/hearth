use crate::reference::Decoded;
use hearth_protocol_mmdebstrap::{
    limits::Limits,
    output::{self, Reader, Seen},
    tail::Cause,
};
use skein_heap::Meter;
use skein_lib::{
    Env, Queue, Time, Wall,
    stream::{Fault, Up},
};
use skein_world::stream::Wire;
/// End normally, at a byte prefix, or with a pipe fault at a byte prefix.
#[derive(Clone, Copy, Debug)]
pub enum End {
    Whole,
    Cut(usize),
    Break(usize),
}
/// Observations made while reading, independent of fragmentation.
#[derive(Debug)]
pub struct Run {
    pub decoded: Decoded,
}
#[must_use]
pub fn end(rng: &mut skein_lib::Rng, bytes: &[u8]) -> End {
    crate::generate::end(rng, bytes)
}
#[must_use]
pub fn read_output(bytes: &[u8], seed: u64, limits: &Limits, end: End) -> Run {
    let mut wire = Wire::new(seed);
    let (bytes, broken) = match end {
        End::Whole => (bytes, false),
        End::Cut(at) => (&bytes[..at.min(bytes.len())], false),
        End::Break(at) => (&bytes[..at.min(bytes.len())], true),
    };
    wire.write(bytes);
    wire.eof = true;
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut above = Queue::with_capacity(output::UP_MAX_OUT.above);
    let mut below = Queue::with_capacity(output::UP_MAX_OUT.below);
    let mut reader = Reader::new(limits);
    output::open(&mut reader, &env, &mut below);
    let mut decoded = Decoded { insts: Vec::new(), fetched: 0, error: None, tail: Vec::new(), cause: Cause::Other };
    for _ in 0..bytes.len() * 10 + 100 {
        if let Some(down) = below.pop() {
            wire.take(down);
        }
        if let Some(mut event) = wire.answer() {
            if broken && matches!(event, Up::End) {
                event = Up::Failed(Fault::Reset);
            }
            output::up(&mut reader, &env, event, &mut above, &mut below);
        }
        match above.pop() {
            Some(Seen::Inst(inst)) => decoded.insts.push(inst),
            Some(Seen::Fetched(bytes)) => decoded.fetched = bytes,
            Some(Seen::Failed(error)) => {
                decoded.error = Some(error);
                finish(&reader, &mut decoded);
                return Run { decoded };
            }
            Some(Seen::Ended) => {
                finish(&reader, &mut decoded);
                if let Some(withdraw) = below.pop() {
                    wire.take(withdraw);
                }
                return Run { decoded };
            }
            None => {}
        }
    }
    panic!("reader did not drain seed {seed}");
}
/// Check each entry point under a heap meter, handing emitted payloads out.
#[must_use]
pub fn read_output_metered(bytes: &[u8], limits: &Limits) -> u64 {
    use skein_lib::{
        Intake,
        stream::{Down, Read},
    };
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    let mut intake = Intake::with_capacity(
        u32::try_from(bytes.len()).expect("bounded fixture value").max(output::largest_demand(limits)),
    );
    intake.append(bytes).expect("bounded fixture value");
    let bound = output::worst_case(limits).expect("bounded fixture value");
    let meter = Meter::new();
    meter.start();
    let mut reader = Reader::new(limits);
    let mut most = meter.check(meter.end(), bound, &"new");
    meter.start();
    output::open(&mut reader, &env, &mut below);
    most = most.max(meter.check(meter.end(), bound, &"open"));
    while let Some(down) = below.pop() {
        let read = match down {
            Down::Demand { read: Read::Nothing, .. } => break,
            Down::Demand { read, .. } => read,
            other @ (Down::Send(_) | Down::Finish) => panic!("unexpected {other:?}"),
        };
        let event = intake.meet(read).map_or(Up::End, Up::Bytes);
        meter.start();
        output::up(&mut reader, &env, event, &mut above, &mut below);
        let step = meter.end();
        drop(above.pop());
        most = most.max(meter.check(step, bound, &"up"));
    }
    meter.start();
    output::close(&mut reader, &mut below);
    most.max(meter.check(meter.end(), bound, &"close"))
}

fn finish(reader: &Reader, decoded: &mut Decoded) {
    decoded.cause = reader.tail().cause().clone();
    decoded.tail = reader.tail().lines().iter().map(|line| line.to_vec()).collect();
}
