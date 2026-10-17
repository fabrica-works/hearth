//! Shared input builders and sweep observations.
use crate::world::{Outcome, Run, Settings};
use hearth_protocol_config::{Fault, json::Waiting, limits::Limits};
#[derive(Default, Debug)]
pub struct Seen {
    bits: u32,
}
impl Seen {
    pub const EVERY_FAULT_AND_ENDING: u32 = 0x3fff;
    pub fn record(&mut self, run: &Run, _settings: &Settings) {
        self.bits |= match &run.outcome {
            Some(Outcome::Decoded) => 1 << 9,
            Some(Outcome::Failed(error)) => match error.fault {
                Fault::Json(_) => 1,
                Fault::Stream(_) => 1 << 1,
                Fault::NotObject => 1 << 2,
                Fault::Duplicate { .. } => 1 << 3,
                Fault::NotInteger => 1 << 4,
                Fault::Range => 1 << 5,
                Fault::Nul => 1 << 6,
                Fault::Keys => 1 << 7,
                Fault::Note => 1 << 8,
                Fault::Shape(_) => panic!("shape is above the machine"),
            },
            None => 0,
        };
        if run.early_end {
            self.bits |= 1 << 10;
        }
        self.bits |= match run.closed_at {
            Waiting::Decode => 1 << 11,
            Waiting::Bytes => 1 << 12,
            Waiting::Close => 1 << 13,
            Waiting::Nothing => 0,
        };
    }
    pub fn assert_fell(&self, expected: u32) {
        assert_eq!(self.bits, expected, "each injected fault and ending fell");
    }
}
#[must_use]
pub fn at_the_limits(limits: &Limits) -> Vec<(Vec<u8>, &'static str)> {
    let width = limits.keys as usize;
    let mut widest =
        format!("{{{}}}", (0..width).map(|n| format!("\"k{n}\":0")).collect::<Vec<_>>().join(",")).into_bytes();
    let mut excess = widest.clone();
    excess.pop();
    excess.extend_from_slice(b",\"extra\":0}");
    let deep = format!(
        "{{\"a\":{}0{}}}",
        "[".repeat(limits.depth.saturating_sub(1) as usize),
        "]".repeat(limits.depth.saturating_sub(1) as usize)
    )
    .into_bytes();
    let past_depth =
        format!("{{\"a\":{}0{}}}", "[".repeat(limits.depth as usize), "]".repeat(limits.depth as usize)).into_bytes();
    let text = format!("{{\"a\":\"{}\"}}", "a".repeat(limits.document.saturating_sub(8) as usize)).into_bytes();
    let past_text = format!("{{\"a\":\"{}\"}}", "a".repeat(limits.document.saturating_sub(7) as usize)).into_bytes();
    let array = format!(
        "{{\"a\":[{}]}}",
        (0..limits.document.saturating_sub(9) / 2).map(|_| "0").collect::<Vec<_>>().join(",")
    )
    .into_bytes();
    widest.shrink_to_fit();
    vec![
        (widest, "widest"),
        (excess, "past width"),
        (deep, "deepest"),
        (past_depth, "past depth"),
        (text, "full string"),
        (past_text, "past length"),
        (array.clone(), "full array"),
        ([array, vec![b' '; 16]].concat(), "past array length"),
    ]
}
