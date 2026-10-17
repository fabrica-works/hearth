//! The shipped binary and its bounded stdout reader on one real ring.
use skein_io::kernel::{Complete, Done, Exit, Fd, Op, Submit};
use skein_lib::{Duration, Queue, Time, Token, Wall};
use skein_shell::{Clock, Host};
use skein_world::{
    Referee,
    end_to_end::{Binary, Command, Mode, Streams},
    real,
};
use std::{ffi::OsStr, path::Path};

#[derive(Debug)]
pub struct Ended {
    pub exit: Option<Exit>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
#[must_use]
pub fn binary(program: &Path, arguments: &[&OsStr], environment: &[(&str, &OsStr)], directory: &Path) -> Ended {
    let mut binary = Binary::start(
        Command {
            program: program.into(),
            arguments: arguments.iter().map(|word| (*word).into()).collect(),
            environment: environment.iter().map(|(name, value)| ((*name).into(), (*value).into())).collect(),
            directory: directory.into(),
        },
        Mode::Pipes,
        65_536,
    )
    .expect("binary starts");
    let streams = binary.take_streams().expect("binary pipes");
    let fds = streams.descriptors();
    let Streams::Pipes { input, output } = streams else { panic!("pipes mode") };
    let mut world = real::World::new(Judge { settled: false });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || Process::Reader(Reader::new(input, output)));
    let outcome = world.run(&Clock::new(), Duration::from_secs(2));
    let mut ended = Ended { exit: None, stdout: Vec::new(), stderr: Vec::new() };
    for process in outcome.procs {
        match process {
            Process::Binary(binary) => {
                ended.exit = binary.exit_status();
                ended.stderr = binary.stderr().to_vec();
            }
            Process::Reader(reader) => ended.stdout = reader.bytes,
        }
    }
    ended
}
struct Reader {
    input: Option<Fd>,
    output: Option<Fd>,
    reading: bool,
    closing: u32,
    bytes: Vec<u8>,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}
impl Reader {
    fn new(input: Fd, output: Fd) -> Self {
        Self {
            input: Some(input),
            output: Some(output),
            reading: false,
            closing: 0,
            bytes: Vec::new(),
            completions: Queue::with_capacity(8),
            submissions: Queue::with_capacity(8),
        }
    }
    fn close(&mut self, fd: Fd) {
        self.closing = self.closing.checked_add(1).expect("two pipes");
        self.submissions.push(Submit { op: Token::new(2), kind: Op::Close { fd } });
    }
}
impl Host for Reader {
    fn iterate(&mut self, _now: Time, _wall: Wall) {
        while let Some(complete) = self.completions.pop() {
            if complete.op == Token::new(2) {
                self.closing = self.closing.checked_sub(1).expect("pending close");
                complete.result.expect("pipe closed");
            } else {
                self.reading = false;
                let Op::PipeRead { buf, .. } = complete.kind else { panic!("stdout read") };
                let Done::Count(count) = complete.result.expect("stdout read completes") else { panic!("read count") };
                self.bytes
                    .extend_from_slice(buf.get(..usize::try_from(count).expect("bounded read")).expect("read fits"));
                assert!(self.bytes.len() <= 65_536, "bounded stdout");
                if count == 0 {
                    let fd = self.output.take().expect("stdout pipe");
                    self.close(fd);
                }
            }
        }
        if let Some(fd) = self.input.take() {
            self.close(fd);
        }
        if !self.reading
            && let Some(fd) = self.output
        {
            self.reading = true;
            self.submissions
                .push(Submit { op: Token::new(1), kind: Op::PipeRead { fd, buf: vec![0; 1024].into_boxed_slice() } });
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, _now: Time) -> bool {
        !self.completions.is_empty() || self.input.is_some() || (!self.reading && self.output.is_some())
    }
    fn next_deadline(&self) -> Option<Time> {
        None
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.input.is_none() && self.output.is_none() && self.closing == 0
    }
    fn operations(&self) -> u32 {
        3
    }
    fn worst_case(&self) -> u64 {
        131_072
    }
}
enum Process {
    Binary(Box<Binary>),
    Reader(Reader),
}
impl Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Binary(binary) => binary.as_ref(),
            Self::Reader(reader) => reader,
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Binary(binary) => binary.as_mut(),
            Self::Reader(reader) => reader,
        }
    }
}
impl Host for Process {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.host_mut().iterate(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host_mut().completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host_mut().submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host().work_pending(now)
    }
    fn next_deadline(&self) -> Option<Time> {
        self.host().next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.host().is_empty()
    }
    fn operations(&self) -> u32 {
        self.host().operations()
    }
    fn worst_case(&self) -> u64 {
        self.host().worst_case()
    }
    fn drain(&mut self) {
        self.host_mut().drain();
    }
}
struct Judge {
    settled: bool,
}
impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, _procs: &mut [Process]) {}
    fn observe(&mut self, _now: Time, procs: &[Process]) {
        self.settled = procs.iter().all(Host::is_empty);
    }
    fn next_deadline(&self) -> Option<Time> {
        None
    }
    fn overdue(&self, _now: Time) -> Option<String> {
        None
    }
    fn passed(&self) -> bool {
        self.settled
    }
}
