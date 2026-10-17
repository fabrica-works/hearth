//! In-process render arguments and startup paths (client/README.md, 3.5).
use crate::{
    Call, Form, Invocation,
    records::startup::{Misuse, Opened, Startup},
};
use std::{
    ffi::{OsStr, OsString},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

#[must_use]
pub fn form(args: &[OsString]) -> Form {
    if args.iter().any(|word| word == "--json") { Form::JsonLines } else { Form::Text }
}
pub fn invocation(
    args: &[OsString],
    xdg_cache_home: Option<&OsStr>,
    home: Option<&OsStr>,
    cwd: &Path,
) -> Result<Invocation, Startup> {
    let mut words = args.iter();
    let command = words.next().map_or(OsStr::new(""), OsString::as_os_str);
    if command != "render" && command != "diff" {
        return Err(usage(Misuse::Command, command));
    }
    let diff = command == "diff";
    let pair = if diff {
        let from = words.next().ok_or_else(|| usage(Misuse::Value, command))?;
        let to = words.next().ok_or_else(|| usage(Misuse::Value, command))?;
        for revision in [from, to] {
            hearth_protocol_git::invocation::revision(revision.as_bytes())
                .map_err(|_| usage(Misuse::Revision, revision))?;
        }
        Some((from, to))
    } else {
        None
    };
    let mut revision = OsStr::new("HEAD");
    let mut repository = None;
    let mut store = None;
    let mut targets = Vec::new();
    while let Some(word) = words.next() {
        match word.as_bytes() {
            b"--json" => {}
            b"--at" | b"--repo" | b"--store" | b"--target" => {
                if (diff && word == "--at") || (!diff && word == "--target") {
                    return Err(usage(Misuse::Option, word));
                }
                let value = words.next().ok_or_else(|| usage(Misuse::Value, word))?;
                match word.as_bytes() {
                    b"--at" => revision = value,
                    b"--repo" => repository = Some(Path::new(value)),
                    b"--target" => targets.push(Box::<[u8]>::from(value.as_bytes())),
                    _ => store = Some(PathBuf::from(value)),
                }
            }
            bytes if bytes.starts_with(b"-") => return Err(usage(Misuse::Option, word)),
            bytes => {
                if diff {
                    return Err(usage(Misuse::Option, word));
                }
                targets.push(Box::<[u8]>::from(bytes));
            }
        }
    }
    hearth_protocol_git::invocation::revision(revision.as_bytes()).map_err(|_| usage(Misuse::Revision, revision))?;
    let store = store
        .or_else(|| default_store(xdg_cache_home, home))
        .ok_or_else(|| unopened(Opened::Store, None, "neither XDG_CACHE_HOME nor HOME is absolute"))?;
    let repository = repository_path(repository, cwd)?;
    Ok(Invocation {
        call: if let Some((from, to)) = pair {
            Call::Diff {
                repository: repository.as_os_str().as_bytes().into(),
                from: from.as_bytes().into(),
                to: to.as_bytes().into(),
                targets: targets.into_boxed_slice(),
            }
        } else {
            Call::Render {
                repository: repository.as_os_str().as_bytes().into(),
                revision: revision.as_bytes().into(),
                targets: targets.into_boxed_slice(),
            }
        },
        form: form(args),
        store: cwd.join(store),
    })
}
fn default_store(xdg: Option<&OsStr>, home: Option<&OsStr>) -> Option<PathBuf> {
    xdg.filter(|path| Path::new(path).is_absolute()).map(|path| Path::new(path).join("hearth/store")).or_else(|| {
        home.filter(|path| Path::new(path).is_absolute()).map(|path| Path::new(path).join(".cache/hearth/store"))
    })
}
fn repository_path(path: Option<&Path>, cwd: &Path) -> Result<PathBuf, Startup> {
    let path = match path {
        Some(path) => {
            let path = cwd.join(path);
            if path.join(".git").exists() { path.join(".git") } else { path }
        }
        None => cwd
            .ancestors()
            .map(|path| path.join(".git"))
            .find(|path| path.exists())
            .ok_or_else(|| unopened(Opened::Repository, Some(cwd), "no enclosing repository"))?,
    };
    std::fs::metadata(&path).map_err(|error| unopened(Opened::Repository, Some(&path), error))?;
    Ok(path)
}
fn usage(misuse: Misuse, word: &OsStr) -> Startup {
    Startup::Usage { misuse, word: word.as_bytes().into() }
}
pub fn unopened(what: Opened, path: Option<&Path>, error: impl std::fmt::Display) -> Startup {
    Startup::Unopened {
        what,
        path: path.map(|path| path.as_os_str().as_bytes().into()),
        error: error.to_string().into_bytes().into(),
    }
}
