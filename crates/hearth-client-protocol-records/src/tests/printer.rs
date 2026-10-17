use super::fixtures;
use crate::{
    form::Form,
    limits::{self, Limits},
    outcome::Outcome,
    printer::{self, Event, Printer, Request, Waiting},
    records::{Record, Terminal},
};
use alloc::boxed::Box;
use skein_lib::{
    Env, List, Queue, Time, Wall, Writer,
    stream::{Down, Fault, Read, Up},
};
struct Machine {
    printer: Printer,
    env: Env<Limits>,
    above: Queue<Event>,
    below: Queue<Down>,
}
struct Printed {
    bytes: Box<[u8]>,
    events: Box<[Event]>,
    largest: usize,
}
impl Machine {
    fn new(form: Form, limits: Limits) -> Self {
        Self {
            printer: Printer::new(form, &limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            above: Queue::with_capacity(printer::UP_MAX_OUT.above.max(printer::DOWN_MAX_OUT.above)),
            below: Queue::with_capacity(printer::UP_MAX_OUT.below.max(printer::DOWN_MAX_OUT.below)),
        }
    }
    fn down(&mut self, request: Request) {
        printer::down(&mut self.printer, &self.env, request, &mut self.above, &mut self.below);
        assert!(self.above.len() <= printer::DOWN_MAX_OUT.above, "down MAX_OUT above");
        assert!(self.below.len() <= printer::DOWN_MAX_OUT.below, "down MAX_OUT below");
    }
    fn up(&mut self, event: Up) {
        printer::up(&mut self.printer, &self.env, event, &mut self.above, &mut self.below);
        assert!(self.above.len() <= printer::UP_MAX_OUT.above, "up MAX_OUT above");
        assert!(self.below.len() <= printer::UP_MAX_OUT.below, "up MAX_OUT below");
    }
    fn print(&mut self, record: Record) -> Printed {
        self.down(Request::Print(record));
        let mut pieces = List::with_capacity(4096);
        let mut events = List::with_capacity(1);
        let mut largest = 0;
        let mut total = 0_usize;
        let mut granted = None;
        for _ in 0..4096_u32 {
            let mut wanted = None;
            while let Some(request) = self.below.pop() {
                match request {
                    Down::Demand { read, room } => {
                        assert_eq!(read, Read::Nothing, "write half only");
                        assert!(room > 0 && room <= self.env.limits.chunk, "room within chunk");
                        wanted = Some(room);
                    }
                    Down::Send(piece) => {
                        assert!(
                            piece.len() <= usize::try_from(granted.take().expect("one Send per grant")).unwrap(),
                            "Send within grant"
                        );
                        largest = largest.max(piece.len());
                        total = total.checked_add(piece.len()).unwrap();
                        pieces.push(piece).unwrap();
                    }
                    Down::Finish => panic!("a record never finishes the stream"),
                }
            }
            while let Some(event) = self.above.pop() {
                events.push(event).unwrap();
            }
            if !events.is_empty() {
                assert_eq!(wanted, None, "no demand after Printed");
                let mut writer = Writer::new(total);
                for piece in pieces.as_slice() {
                    writer.put(piece).unwrap();
                }
                return Printed { bytes: writer.finish(), events: events.into_boxed(), largest };
            }
            granted = Some(wanted.expect("printing demands room"));
            self.up(Up::Room);
        }
        panic!("a fixture finishes within 4096 pieces");
    }
}
fn terminal() -> Record {
    Record::Terminal(Terminal { outcome: Outcome::Done })
}
#[test]
fn a_record_goes_down_in_pieces_of_at_most_a_chunk() {
    let mut machine = Machine::new(Form::Text, Limits { chunk: 16, ..limits::SHIPPED });
    let printed = machine.print(Record::Spec(fixtures::x1_prod()));
    assert_eq!(&*printed.bytes, fixtures::X1_PROD_TEXT.as_bytes(), "spec lines whole and ordered");
    assert!(printed.largest <= 16, "no piece past the chunk");
    assert_eq!(&*printed.events, &[Event::Printed], "one Printed answers the record");
}
#[test]
fn only_the_answers_first_record_follows_the_version_line() {
    let mut machine = Machine::new(Form::JsonLines, limits::SHIPPED);
    let first = machine.print(terminal());
    let second = machine.print(terminal());
    assert_eq!(
        &*first.bytes, b"{\"record\":\"version\",\"client\":1}\n{\"record\":\"outcome\",\"outcome\":\"done\"}\n",
        "version first"
    );
    assert_eq!(&*second.bytes, b"{\"record\":\"outcome\",\"outcome\":\"done\"}\n", "version only once");
}
#[test]
fn a_record_that_prints_nothing_is_printed_at_once() {
    let mut machine = Machine::new(Form::Text, limits::SHIPPED);
    machine.down(Request::Print(terminal()));
    assert_eq!(machine.above.pop(), Some(Event::Printed), "immediate terminal");
    assert_eq!(machine.below.pop(), None, "no room needed");
    assert_eq!(machine.printer.waiting(), Waiting::Above, "ready for next record");
}
#[test]
fn finish_ends_the_stream_once_nothing_is_printing() {
    let mut machine = Machine::new(Form::Text, limits::SHIPPED);
    machine.print(terminal());
    machine.down(Request::Finish);
    assert_eq!(machine.below.pop(), Some(Down::Finish), "one stream finish");
    assert_eq!(machine.below.pop(), None, "nothing else");
    assert_eq!(machine.printer.waiting(), Waiting::Close, "finish awaits close");
}
#[test]
fn a_failed_stream_fails_the_record_being_printed() {
    let mut machine = Machine::new(Form::Text, limits::SHIPPED);
    machine.down(Request::Print(Record::Spec(fixtures::x1_prod())));
    machine.below.pop().expect("outstanding demand");
    machine.up(Up::Failed(Fault::Other));
    assert_eq!(machine.above.pop(), Some(Event::Failed(Fault::Other)), "one record failure");
    assert_eq!(machine.printer.waiting(), Waiting::Close, "only close follows failure");
}
#[test]
fn close_withdraws_the_demand_and_answers_closed() {
    let mut machine = Machine::new(Form::Text, limits::SHIPPED);
    machine.down(Request::Print(Record::Spec(fixtures::x1_prod())));
    machine.below.pop().expect("outstanding demand");
    machine.down(Request::Close);
    assert_eq!(machine.below.pop(), Some(Down::Demand { read: Read::Nothing, room: 0 }), "withdraw room");
    assert_eq!(machine.above.pop(), Some(Event::Closed), "one close answer");
    machine.up(Up::Room);
    assert_eq!(machine.printer.waiting(), Waiting::Nothing, "late room discarded");
}
