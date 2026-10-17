//! The shipped builder host and its startup directories (shell.md, 6).
#![forbid(unsafe_code)]
pub use hearth_service_builder as service;
use service::{Configuration, Roots, Service};
pub use service::{Limits, domain::vocabulary::Call, protocol::Form};
use skein_io::kernel::{Complete, Exit, Fd, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::{Host, OutputKind};
use std::path::PathBuf;
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Invocation {
    pub call: Call,
    pub form: Form,
    pub store: PathBuf,
}
#[derive(Debug)]
pub struct Client {
    service: Service,
    limits: Limits,
    store: Option<Fd>,
}
impl Client {
    pub fn open(invocation: Invocation, limits: &Limits, output: Fd) -> Result<Self, std::io::Error> {
        let kind = skein_shell::prepare_output(output)
            .map_err(|error| std::io::Error::other(format!("invalid output: {error:?}")))?;
        let store = skein_shell::make_root(&invocation.store, 0o700).map_err(std::io::Error::from_raw_os_error)?;
        let mut service =
            Service::new(&Configuration { limits: *limits }, Roots { store }, invocation.call, invocation.form);
        let adopted = match kind {
            OutputKind::Pipe => service.adopt_output_pipe(output),
            OutputKind::Append => service.adopt_output_file(output),
        };
        if adopted.is_err() {
            skein_shell::close_keeper_fd(store);
            return Err(std::io::Error::other("no output slot"));
        }
        Ok(Self { service, limits: *limits, store: Some(store) })
    }
}
impl Host for Client {
    fn iterate(&mut self, now: Time, wall: Wall) {
        service::iterate(&mut self.service, now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.service.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.service.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.service.work_pending(now)
    }
    fn next_deadline(&self) -> Option<Time> {
        self.service.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.service.is_empty() && self.store.is_none()
    }
    fn exit(&self) -> Option<Exit> {
        if self.is_empty() {
            Some(Exit::Code(self.service.code().expect("settled client has its terminal code")))
        } else {
            None
        }
    }
    fn operations(&self) -> u32 {
        service::operations(&self.limits).expect("shipped io operation count fits")
    }
    fn worst_case(&self) -> u64 {
        0
    } // No heap claim until hearth 28.
    fn drain(&mut self) {
        if self.service.is_empty()
            && let Some(store) = self.store.take()
        {
            skein_shell::close_keeper_fd(store);
        }
    }
}
#[must_use]
pub const fn limits() -> Limits {
    service::limits::SHIPPED
}
