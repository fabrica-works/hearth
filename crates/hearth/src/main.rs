//! The in-process client's shell (client/README.md, 3.5).
use hearth::{
    Client, argv,
    records::{
        self,
        records::Record,
        startup::{Opened, Startup},
    },
};
use skein_io::kernel::{Exit, Fd};
use skein_shell::{Clock, Config, Kernel};
use std::{ffi::OsString, io::Write, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let form = argv::form(&args);
    let xdg = std::env::var_os("XDG_CACHE_HOME");
    let home = std::env::var_os("HOME");
    let code = match run(&args, xdg.as_deref(), home.as_deref()) {
        Ok(code) => code,
        Err(startup) => {
            let outcome = startup.outcome();
            let mut stderr = std::io::stderr().lock();
            let limits = records::limits::SHIPPED;
            if let Ok(bytes) = records::form::print(form, &Record::Startup(startup), None, &limits) {
                let _written = stderr.write_all(&bytes);
            }
            records::outcome::code(outcome)
        }
    };
    ExitCode::from(code)
}
fn run(args: &[OsString], xdg: Option<&std::ffi::OsStr>, home: Option<&std::ffi::OsStr>) -> Result<u8, Startup> {
    let cwd = std::env::current_dir().map_err(|error| argv::unopened(Opened::Repository, None, error))?;
    let invocation = argv::invocation(args, xdg, home, &cwd)?;
    let seed = hearth::seed()?;
    let limits = hearth::limits();
    let store = invocation.store.clone();
    let mut kernel =
        Kernel::open(Config { operations: hearth::service::operations(&limits).expect("shipped operations fit") })
            .map_err(|error| argv::unopened(Opened::Store, Some(&store), format!("{error:?}")))?;
    let mut client = Client::open(invocation, &limits, Fd::new(1), seed)
        .map_err(|error| argv::unopened(Opened::Store, Some(&store), error))?;
    match skein_shell::drive(&mut kernel, &Clock::new(), &mut client) {
        Exit::Code(code) => Ok(code),
        Exit::Signal(_) => Ok(75),
    }
}
