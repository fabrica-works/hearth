//! Simulate's argument vector and sources.list (render.md, 5.2).
use crate::limits::Limits;
use alloc::boxed::Box;
use skein_lib::{List, Writer, bytes};

/// Debian Policy 5.6.1's name, bounded by `Limits::name`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name(Box<[u8]>);
impl Name {
    #[must_use]
    pub fn new(bytes: &[u8], limits: &Limits) -> Option<Name> {
        if bytes.len() < 2 || bytes.len() > usize::try_from(limits.name).ok()? {
            return None;
        }
        if !bytes.first()?.is_ascii_lowercase() && !bytes.first()?.is_ascii_digit() {
            return None;
        }
        for b in bytes {
            if !b.is_ascii_lowercase() && !b.is_ascii_digit() && !b"+-.".contains(b) {
                return None;
            }
        }
        Some(Name(bytes::copy_of(bytes)))
    }
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
/// Snapshot timestamp, digits checked, not its date (format.md, 5.3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Timestamp([u8; 16]);
impl Timestamp {
    #[must_use]
    pub fn new(bytes: &[u8]) -> Option<Timestamp> {
        let value: [u8; 16] = bytes.try_into().ok()?;
        for (i, b) in value.iter().enumerate() {
            let good = match i {
                8 => *b == b'T',
                15 => *b == b'Z',
                _ => b.is_ascii_digit(),
            };
            if !good {
                return None;
            }
        }
        Some(Timestamp(value))
    }
}
/// Components in the format's fixed order, main always present.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Component {
    Main,
    Contrib,
    NonFree,
    NonFreeFirmware,
}
impl Component {
    pub(crate) fn bytes(self) -> &'static [u8] {
        match self {
            Self::Main => b"main",
            Self::Contrib => b"contrib",
            Self::NonFree => b"non-free",
            Self::NonFreeFirmware => b"non-free-firmware",
        }
    }
}
/// The packages part's sources, field for field (render.md, 4.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sources {
    pub suite: Name,
    pub architecture: Name,
    pub components: Box<[Component]>,
    pub archive: Timestamp,
    pub updates: bool,
    pub security: Option<Timestamp>,
}
/// Bootstrap variant: apt for metal and VM, no container resolution in M1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    Apt,
}
/// The packages part's apt options.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Apt {
    pub install_recommends: bool,
}
/// Resolution's key inputs (render.md, 5.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Simulate {
    pub sources: Sources,
    pub request: Box<[Name]>,
    pub variant: Variant,
    pub apt: Apt,
}
/// A printable HTTP proxy URI without apt's quoting or statement bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proxy(Box<[u8]>);
impl Proxy {
    #[must_use]
    pub fn new(bytes: &[u8], limits: &Limits) -> Option<Proxy> {
        if !bytes.starts_with(b"http://") || bytes.len() > usize::try_from(limits.path).ok()? {
            return None;
        }
        for b in bytes {
            if !(32..=126).contains(b) || b"\";\\{}".contains(b) {
                return None;
            }
        }
        Some(Proxy(bytes::copy_of(bytes)))
    }
}
/// A source line's archive URI, suite and ordered components (render.md, 5.4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub uri: Box<[u8]>,
    pub suite: Box<[u8]>,
    pub components: Box<[Component]>,
}
/// Arguments after argv[0], complete environment and source file bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invocation {
    pub args: Box<[Box<[u8]>]>,
    pub env: Box<[Box<[u8]>]>,
    pub sources_list: Box<[u8]>,
}
/// Components, request or scratch outside their accepted forms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VectorError {
    Components,
    Request,
    Scratch,
}
pub const PROGRAM: &[u8] = b"/usr/bin/mmdebstrap";
pub const ARCHIVE: &[u8] = b"http://snapshot.debian.org/archive/debian/";
pub const SECURITY: &[u8] = b"http://snapshot.debian.org/archive/debian-security/";

fn join(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded vector's size fits usize");
    }
    let mut out = Writer::new(length);
    for part in parts {
        out.put(part).expect("summed vector length");
    }
    out.finish()
}
fn append(args: &mut List<Box<[u8]>>, bytes: Box<[u8]>) {
    args.push(bytes).expect("argument count bounded by request plus fixed options");
}
/// Validate components and expand enabled pockets, in their fixed order.
pub fn lines(sources: &Sources) -> Result<Box<[Line]>, VectorError> {
    if sources.components.first() != Some(&Component::Main) {
        return Err(VectorError::Components);
    }
    for pair in sources.components.windows(2) {
        if pair.first() >= pair.get(1) {
            return Err(VectorError::Components);
        }
    }
    let mut out = List::with_capacity(3);
    let uri = join(&[ARCHIVE, &sources.archive.0, b"/"]);
    out.push(Line { uri, suite: bytes::copy_of(sources.suite.as_bytes()), components: sources.components.clone() })
        .expect("one archive line");
    if sources.updates {
        let uri = join(&[ARCHIVE, &sources.archive.0, b"/"]);
        out.push(Line {
            uri,
            suite: join(&[sources.suite.as_bytes(), b"-updates"]),
            components: sources.components.clone(),
        })
        .expect("one updates line");
    }
    if let Some(time) = &sources.security {
        let uri = join(&[SECURITY, &time.0, b"/"]);
        out.push(Line {
            uri,
            suite: join(&[sources.suite.as_bytes(), b"-security"]),
            components: sources.components.clone(),
        })
        .expect("one security line");
    }
    Ok(out.into_boxed())
}
/// Encode one deb line per source, ending each with LF.
#[must_use]
pub fn sources_list(lines: &[Line]) -> Box<[u8]> {
    let mut parts = List::with_capacity(u32::try_from(lines.len()).expect("source count fits u32").saturating_mul(14));
    for line in lines {
        append(&mut parts, join(&[b"deb ", &line.uri, b" ", &line.suite]));
        for component in &line.components {
            append(&mut parts, join(&[b" ", component.bytes()]));
        }
        append(&mut parts, bytes::copy_of(b"\n"));
    }
    let mut length = 0_usize;
    for part in &parts {
        length = length.checked_add(part.len()).expect("bounded source list length");
    }
    let mut out = Writer::new(length);
    for part in &parts {
        out.put(part).expect("summed source list length");
    }
    out.finish()
}
/// Build simulate's vector; names are refused at construction, never quoted.
pub fn simulate(
    simulate: &Simulate,
    scratch: &[u8],
    proxy: Option<&Proxy>,
    limits: &Limits,
) -> Result<Invocation, VectorError> {
    let path = usize::try_from(limits.path).expect("u32 fits usize");
    if !scratch.starts_with(b"/")
        || scratch.ends_with(b"/")
        || scratch.contains(&0)
        || scratch.len().saturating_add(13) > path
    {
        return Err(VectorError::Scratch);
    }
    if simulate.request.len() > usize::try_from(limits.request).expect("u32 fits usize") {
        return Err(VectorError::Request);
    }
    for pair in simulate.request.windows(2) {
        if pair.first() >= pair.get(1) {
            return Err(VectorError::Request);
        }
    }
    let lines = lines(&simulate.sources)?;
    let capacity = limits.request.checked_add(12).ok_or(VectorError::Request)?;
    let mut args = List::with_capacity(capacity);
    for fixed in [b"--verbose".as_slice(), b"--simulate", b"--mode=chrootless", b"--variant=apt"] {
        append(&mut args, bytes::copy_of(fixed));
    }
    append(&mut args, join(&[b"--architectures=", simulate.sources.architecture.as_bytes()]));
    includes(&mut args, &simulate.request, limits)?;
    append(&mut args, bytes::copy_of(b"--aptopt=Acquire::Check-Valid-Until \"false\""));
    let recommends = if simulate.apt.install_recommends { b"true".as_slice() } else { b"false" };
    append(&mut args, join(&[b"--aptopt=APT::Install-Recommends \"", recommends, b"\""]));
    if let Some(proxy) = proxy {
        append(&mut args, join(&[b"--aptopt=Acquire::http::Proxy \"", &proxy.0, b"\""]));
    }
    append(&mut args, bytes::copy_of(b"--skip=cleanup/apt/lists"));
    append(&mut args, bytes::copy_of(simulate.sources.suite.as_bytes()));
    append(&mut args, join(&[scratch, b"/root"]));
    append(&mut args, join(&[scratch, b"/sources.list"]));
    let mut env = List::with_capacity(3);
    append(&mut env, bytes::copy_of(b"LC_ALL=C"));
    append(&mut env, bytes::copy_of(b"PATH=/usr/sbin:/usr/bin:/sbin:/bin"));
    append(&mut env, join(&[b"TMPDIR=", scratch]));
    Ok(Invocation { args: args.into_boxed(), env: env.into_boxed(), sources_list: sources_list(&lines) })
}
fn includes(args: &mut List<Box<[u8]>>, request: &[Name], limits: &Limits) -> Result<(), VectorError> {
    let max = usize::try_from(limits.include).expect("u32 fits usize");
    let mut part = bytes::copy_of(b"--include=");
    for name in request {
        let separator = if part.len() == 10 { b"".as_slice() } else { b"," };
        if part.len().saturating_add(separator.len()).saturating_add(name.as_bytes().len()) > max {
            if part.len() == 10 {
                return Err(VectorError::Request);
            }
            append(args, part);
            part = join(&[b"--include=", name.as_bytes()]);
            if part.len() > max {
                return Err(VectorError::Request);
            }
        } else {
            part = join(&[&part, separator, name.as_bytes()]);
        }
    }
    if part.len() > 10 {
        append(args, part);
    }
    Ok(())
}
