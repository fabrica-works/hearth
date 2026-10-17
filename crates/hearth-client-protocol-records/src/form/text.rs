//! The stable text form (render.md, 9.2), measured before allocation.
use crate::{
    form::Refusal,
    records::{Record, Type},
};
use alloc::boxed::Box;
use skein_lib::Writer;
#[derive(Debug)]
pub(crate) struct Lines {
    sink: Sink,
    column: usize,
    length: Option<usize>,
}
#[derive(Debug)]
enum Sink {
    Measure,
    Write(Writer),
}
impl Lines {
    fn measure() -> Self {
        Self { sink: Sink::Measure, column: 0, length: Some(0) }
    }
    fn write(length: usize) -> Self {
        Self { sink: Sink::Write(Writer::new(length)), column: 0, length: Some(0) }
    }
    pub(crate) fn put(&mut self, bytes: &[u8]) {
        self.length = match self.length {
            Some(n) => n.checked_add(bytes.len()),
            None => None,
        };
        for byte in bytes {
            self.column = if *byte == b'\n' { 0 } else { self.column.saturating_add(1) };
        }
        match &mut self.sink {
            Sink::Measure => {}
            Sink::Write(writer) => writer.put(bytes).expect("measured text fits"),
        }
    }
    pub(crate) fn pad_to(&mut self, column: usize) {
        for _ in self.column..column {
            self.put(b" ");
        }
    }
    pub(crate) fn hex12(&mut self, digest: &[u8; 32]) {
        for byte in digest.iter().take(6) {
            self.put(&[hex(byte >> 4), hex(byte & 15)]);
        }
    }
    pub(crate) fn decimal(&mut self, number: u64) {
        self.put(skein_lib::Decimal::of(number).as_bytes());
    }
    pub(crate) fn mode(&mut self, mode: u16) {
        for shift in [9_u32, 6, 3, 0] {
            self.put(&[b'0'.saturating_add(u8::try_from((mode >> shift) & 7).expect("octal digit"))]);
        }
    }
}
fn hex(n: u8) -> u8 {
    if n < 10 { b'0'.saturating_add(n) } else { b'a'.saturating_add(n.saturating_sub(10)) }
}
pub(crate) fn print(record: &Record, after: Option<Type>, most: u32) -> Result<Box<[u8]>, Refusal> {
    let mut lines = Lines::measure();
    render(&mut lines, record, after);
    let length = lines.length.ok_or(Refusal::TooLong)?;
    if length > usize::try_from(most).expect("u32 fits usize") {
        return Err(Refusal::TooLong);
    }
    let mut lines = Lines::write(length);
    render(&mut lines, record, after);
    match lines.sink {
        Sink::Write(writer) => Ok(writer.finish()),
        Sink::Measure => unreachable!("writing pass"),
    }
}
fn render(lines: &mut Lines, record: &Record, after: Option<Type>) {
    match record {
        Record::Spec(spec) => {
            if after.is_some() {
                lines.put(b"\n");
            }
            let mut column = 13;
            for part in &*spec.parts {
                column = column.max(part.record.name().len().saturating_add(2));
            }
            spec.meta.text(lines, column);
            lines.put(b"spec");
            lines.pad_to(column);
            lines.hex12(&spec.digest);
            lines.put(b"\n");
            for part in &*spec.parts {
                lines.put(part.record.name());
                lines.pad_to(column);
                lines.put(b"part ");
                lines.hex12(&part.digest);
                lines.put(b"\n");
                part.record.text(lines);
            }
            if let Some(resolution) = &spec.resolution {
                match resolution {
                    crate::records::Resolution::Needed { .. } => {}
                    crate::records::Resolution::Known { keys, .. } => {
                        lines.put(b"keys");
                        lines.pad_to(column);
                        lines.put(b"packages ");
                        lines.hex12(&keys.packages);
                        lines.put(b"   software ");
                        lines.hex12(&keys.software);
                        lines.put(b"   config ");
                        lines.hex12(&keys.config);
                        lines.put(b"\n");
                    }
                }
            }
        }
        Record::Change(change) => change.text(lines),
        Record::Terminal(terminal) => {
            if let Some(tally) = &terminal.diff {
                tally.text(lines);
            }
        }
    }
}
