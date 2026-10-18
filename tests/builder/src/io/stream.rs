//! A demanded stream's intake and bounded writes on real machine pipes.
use super::Below;
use crate::fixtures::{self, Fault};
use hearth_fake_machine::linux::{Handle, Op, Pid, Refusal, Reply};
use hearth_protocol_builder::Told;
use skein_io::{self as io, kernel::Way};
use skein_lib::{
    Intake, Token,
    stream::{Down, Fault as StreamFault, Read, Up},
};
use std::collections::VecDeque;

#[derive(Debug)]
pub(super) struct Pipe {
    pub handle: Handle,
    pub child: Pid,
    pub fd: u32,
    pub way: Way,
    pub ended: bool,
    pub closed: bool,
    demand: Option<(Read, u32)>,
    intake: Intake,
    sending: VecDeque<(Box<[u8]>, usize)>,
    finishing: bool,
    read: u32,
}
impl Pipe {
    pub fn new(handle: Handle, child: Pid, fd: u32, way: Way) -> Self {
        Self {
            handle,
            child,
            fd,
            way,
            ended: false,
            closed: false,
            demand: None,
            intake: Intake::with_capacity(hearth::limits().io.intake),
            sending: VecDeque::new(),
            finishing: false,
            read: 0,
        }
    }
}
impl Below {
    pub(super) fn stream(&mut self, token: Token, down: Down) {
        if token == fixtures::PROTOCOL_OUTPUT || token == fixtures::PROTOCOL_ERROR {
            match down {
                Down::Demand { room, .. } => {
                    if room > 0 {
                        self.events.push_back(Told::Io(io::Event::Stream { owner: token, up: Up::Room }));
                    }
                }
                Down::Send(bytes) => {
                    if token == fixtures::PROTOCOL_OUTPUT {
                        self.stdout.extend_from_slice(&bytes);
                    } else {
                        self.stderr.extend_from_slice(&bytes);
                    }
                }
                Down::Finish => {}
            }
            return;
        }
        let pipe = self.pipes.get_mut(&token).expect("an issued pipe");
        match down {
            Down::Demand { read, room } => {
                assert!(pipe.demand.is_none() || read == Read::Nothing, "one outstanding demand");
                pipe.demand = if read == Read::Nothing && room == 0 { None } else { Some((read, room)) };
            }
            Down::Send(bytes) => {
                pipe.sending.push_back((bytes, 0));
            }
            Down::Finish => {
                pipe.finishing = true;
            }
        }
    }
    pub(super) fn step_pipes(&mut self) {
        let tokens: Vec<_> = self.pipes.keys().copied().collect();
        for token in tokens {
            let mut pipe = self.pipes.remove(&token).expect("issued pipe");
            self.flush(&mut pipe, token);
            if let Some((read, room)) = pipe.demand {
                if room > 0
                    && pipe.sending.iter().map(|(bytes, at)| bytes.len() - at).sum::<usize>()
                        + usize::try_from(room).expect("room fits")
                        <= usize::try_from(hearth::limits().io.output).expect("output fits")
                {
                    pipe.demand = None;
                    self.events.push_back(Told::Io(io::Event::Stream { owner: token, up: Up::Room }));
                } else if read != Read::Nothing {
                    self.read_pipe(&mut pipe, token, read);
                }
            }
            self.pipes.insert(token, pipe);
        }
    }
    fn flush(&mut self, pipe: &mut Pipe, token: Token) {
        if pipe.finishing && pipe.sending.is_empty() && !pipe.closed {
            self.call(Op::Close { handle: pipe.handle }).expect("finish stdin");
            pipe.closed = true;
        }
        let Some((bytes, offset)) = pipe.sending.front_mut() else { return };
        match self.call(Op::Send { pipe: pipe.handle, bytes: bytes[*offset..].into() }) {
            Ok(Reply::Sent(count)) => {
                *offset += usize::try_from(count).expect("pipe size fits");
                if *offset == bytes.len() {
                    pipe.sending.pop_front();
                }
            }
            Err(Refusal::BrokenPipe) => {
                pipe.sending.clear();
                pipe.demand = None;
                self.events.push_back(Told::Io(io::Event::Stream { owner: token, up: Up::Failed(StreamFault::Reset) }));
            }
            reply => panic!("send writes what fits or reports the closed peer: {reply:?}"),
        }
    }
    fn read_pipe(&mut self, pipe: &mut Pipe, token: Token, read: Read) {
        if let Some(bytes) = pipe.intake.meet(read) {
            pipe.demand = None;
            self.events.push_back(Told::Io(io::Event::Stream { owner: token, up: Up::Bytes(bytes) }));
            return;
        }
        if pipe.ended {
            return;
        }
        let mut most =
            u32::try_from(self.rng.between(1, fixtures::PIPE_GRAIN)).expect("grain fits").min(pipe.intake.room());
        if let Some(Fault::KilledAt { stdout_bytes }) = self.fault
            && !self.fault_fell
            && pipe.fd == 1
        {
            most = most.min(stdout_bytes.saturating_sub(pipe.read));
            if most == 0 {
                self.kill(pipe.child);
                self.fault_fell = true;
                return;
            }
        }
        match self.call(Op::Take { pipe: pipe.handle, most }) {
            Ok(Reply::Took(bytes)) => {
                pipe.read += u32::try_from(bytes.len()).expect("bounded tool output");
                pipe.intake.append(&bytes).expect("read within intake room");
                if let Some(bytes) = pipe.intake.meet(read) {
                    pipe.demand = None;
                    self.events.push_back(Told::Io(io::Event::Stream { owner: token, up: Up::Bytes(bytes) }));
                }
            }
            Ok(Reply::Ended) => {
                pipe.ended = true;
                self.ended_pipes += 1;
                self.events.push_back(Told::Io(io::Event::Stream { owner: token, up: Up::End }));
            }
            Ok(Reply::Empty) => {}
            reply => panic!("take gives bytes, waits or ends: {reply:?}"),
        }
    }
}
