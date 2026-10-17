//! Real git with the same environment and vectors as hearth (render.md, 2.2).
use hearth_protocol_git::invocation::{self, Exit};
use hearth_recordings::Recording;
use std::{
    io::Write,
    os::unix::{ffi::OsStrExt, process::ExitStatusExt},
    path::Path,
    process::{Command, Stdio},
};
#[derive(Debug)]
pub struct Run {
    pub exit: Exit,
    pub captured: Recording,
}
impl Run {
    #[must_use]
    pub fn stderr_text(&self) -> String {
        String::from_utf8_lossy(&self.captured.stderr).into_owned()
    }
    #[must_use]
    pub fn recording(&self) -> Recording {
        self.captured.clone()
    }
}
#[must_use]
pub fn git(args: &[Vec<u8>], dir: &Path, stdin: Option<&[u8]>, env: &[(&str, &str)]) -> Run {
    let mut command = Command::new(std::ffi::OsStr::from_bytes(invocation::GIT));
    command.env_clear().current_dir(dir).args(args.iter().map(|a| std::ffi::OsStr::from_bytes(a)));
    let mut environment: Vec<_> = invocation::environment().iter().map(|b| b.to_vec()).collect();
    for pair in &environment {
        let at = pair.iter().position(|b| *b == b'=').expect("environment pair");
        command.env(std::ffi::OsStr::from_bytes(&pair[..at]), std::ffi::OsStr::from_bytes(&pair[at + 1..]));
    }
    command.envs(env.iter().copied());
    environment.extend(env.iter().map(|(k, v)| format!("{k}={v}").into_bytes()));
    command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().expect("conformance requires /usr/bin/git");
    let mut pipe = child.stdin.take().expect("git stdin");
    if let Some(input) = stdin {
        pipe.write_all(input).expect("git stdin written whole");
    }
    drop(pipe);
    let output = child.wait_with_output().expect("git output");
    let exit = output.status.code().map_or_else(
        || Exit::Signal(u32::try_from(output.status.signal().expect("signal")).expect("positive signal")),
        |code| Exit::Code(u8::try_from(code).expect("git exit byte")),
    );
    let status = match exit {
        Exit::Code(code) => code.to_string(),
        Exit::Signal(signal) => format!("signal {signal}"),
    };
    let mut vector = vec![invocation::GIT.to_vec()];
    vector.extend_from_slice(args);
    Run {
        exit,
        captured: Recording {
            version: "".into(),
            argv: vector,
            env: environment,
            dir: dir.as_os_str().as_encoded_bytes().to_vec(),
            stdin: stdin.map(<[u8]>::to_vec),
            stdout: output.stdout,
            stderr: output.stderr,
            exit: status.into(),
            expect: None,
        },
    }
}
