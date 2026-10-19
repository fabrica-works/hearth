//! The shipped builder host and its startup directories (shell.md, 6).

pub mod argv;
pub use hearth_client_protocol_records as records;
pub use hearth_service_builder as service;
use service::{Configuration, Roots, Service};
pub use service::{Limits, domain::vocabulary::Call, protocol::Form};
use skein_io::kernel::{Complete, Exit, Fd, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::{Host, OutputKind};
use std::{
    io::Write,
    os::unix::{ffi::OsStrExt, fs::DirBuilderExt},
    path::PathBuf,
};
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Invocation {
    pub proxy: Option<Box<[u8]>>,
    pub call: Call,
    pub form: Form,
    pub store: PathBuf,
}
#[derive(Debug)]
pub struct Client {
    service: Service,
    limits: Limits,
    store: Option<Fd>,
    cache: Option<Fd>,
    resolutions: Option<Fd>,
    staging: Option<Fd>,
}
impl Client {
    pub fn open(invocation: Invocation, limits: &Limits, output: Fd, seed: u64) -> Result<Self, std::io::Error> {
        umask(0o022);
        let kind = skein_shell::prepare_output(output)
            .map_err(|error| std::io::Error::other(format!("invalid output: {error:?}")))?;
        std::fs::DirBuilder::new().recursive(true).mode(0o700).create(&invocation.store)?;
        let store = skein_shell::make_root(&invocation.store, 0o700).map_err(std::io::Error::from_raw_os_error)?;
        let mut opened = vec![store];
        for name in ["cache", "resolutions", "staging"] {
            match skein_shell::make_root(&invocation.store.join(name), 0o700) {
                Ok(root) => opened.push(root),
                Err(error) => {
                    for root in opened {
                        skein_shell::close_keeper_fd(root);
                    }
                    return Err(std::io::Error::from_raw_os_error(error));
                }
            }
        }
        let mut opened = opened.into_iter();
        let store = opened.next().expect("store");
        let cache = opened.next().expect("cache");
        let resolutions = opened.next().expect("resolutions");
        let staging = opened.next().expect("staging");
        let client = Self::with_output(
            Configuration {
                proxy: invocation.proxy,
                limits: *limits,
                uid: skein_shell::effective_user(),
                store: invocation.store.as_os_str().as_bytes().into(),
            },
            Roots { store, cache, resolutions, staging },
            invocation.call,
            invocation.form,
            output,
            kind,
            seed,
        );
        let mut client = match client {
            Ok(client) => client,
            Err(error) => {
                for root in [staging, resolutions, cache, store] {
                    skein_shell::close_keeper_fd(root);
                }
                return Err(error);
            }
        };
        client.store = Some(store);
        client.cache = Some(cache);
        client.resolutions = Some(resolutions);
        client.staging = Some(staging);
        Ok(client)
    }
    /// Start over caller-owned directories and an append output (domain/testing.md, 2.6).
    #[must_use]
    pub fn with_roots(
        configuration: Configuration,
        roots: Roots,
        call: Call,
        form: Form,
        output: Fd,
        seed: u64,
    ) -> Self {
        Self::with_output(configuration, roots, call, form, output, OutputKind::Append, seed)
            .expect("the test configuration reserves its output slot")
    }
    fn with_output(
        configuration: Configuration,
        roots: Roots,
        call: Call,
        form: Form,
        output: Fd,
        kind: OutputKind,
        seed: u64,
    ) -> Result<Self, std::io::Error> {
        let mut service = Service::new(&configuration, roots, call, form, seed);
        match kind {
            OutputKind::Pipe => service.adopt_output_pipe(output),
            OutputKind::Append => service.adopt_output_file(output),
        }
        .map_err(|_| std::io::Error::other("no output slot"))?;
        let limits = configuration.limits;
        drop(configuration);
        Ok(Self { service, limits, store: None, cache: None, resolutions: None, staging: None })
    }
    /// Adopt a caller-opened append descriptor for store progress.
    pub fn adopt_error_file(&mut self, fd: Fd) -> Result<(), Fd> {
        self.service.adopt_error_file(fd)
    }
    pub fn adopt_error(&mut self, fd: Fd) -> Result<(), std::io::Error> {
        let kind = skein_shell::prepare_output(fd)
            .map_err(|error| std::io::Error::other(format!("invalid error output: {error:?}")))?;
        match kind {
            OutputKind::Pipe => self.service.adopt_error_pipe(fd),
            OutputKind::Append => self.service.adopt_error_file(fd),
        }
        .map_err(|_| std::io::Error::other("no error output slot"))
    }
    #[must_use]
    pub fn take_startup(&mut self) -> Option<Box<[u8]>> {
        self.service.take_startup()
    }
}
impl Host for Client {
    fn state_digest(&self) -> Option<[u8; 32]> {
        Some(skein_lib::state_digest(self.service.domain()))
    }
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
        service::worst_case(&self.limits).expect("configured builder worst case fits")
    }
    fn drain(&mut self) {
        if let Some(bytes) = self.service.take_startup() {
            let _written = std::io::stderr().lock().write_all(&bytes);
        }
        if self.service.is_empty() {
            for root in [self.resolutions.take(), self.staging.take()].into_iter().flatten() {
                skein_shell::close_keeper_fd(root);
            }
        }
        if self.service.is_empty()
            && let Some(cache) = self.cache.take()
        {
            skein_shell::close_keeper_fd(cache);
        }
        if self.service.is_empty()
            && let Some(store) = self.store.take()
        {
            skein_shell::close_keeper_fd(store);
        }
    }
}
/// Set the process's file creation mask at startup (domain/store.md, 2.2).
#[expect(unsafe_code, reason = "the shell owns the process creation mask")]
pub fn umask(mask: u16) {
    // SAFETY: umask accepts every mode value and holds no pointers or resources.
    unsafe {
        libc::umask(u32::from(mask));
    }
}
#[must_use]
pub const fn limits() -> Limits {
    service::limits::SHIPPED
}

/// Draw the cache temporary generator's seed at process startup (domain/store.md, 2.1).
pub fn seed() -> Result<u64, records::startup::Startup> {
    skein_shell::seed().map_err(|error| {
        argv::unopened(records::startup::Opened::Store, None, std::io::Error::from_raw_os_error(error))
    })
}
#[cfg(test)]
mod tests;
