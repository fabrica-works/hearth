use super::{ERROR, Layer, OUTPUT, REQUEST};
use crate::{Asked, Told, builder, limits, render};
use hearth_protocol_mmdebstrap::vector;
use skein_io::{self as io, file, kernel::Exit};
use skein_lib::{
    Duration, List, Token, bytes,
    stream::{Down, Held, Read, Up},
};
pub const KEY: render::vocabulary::Digest = render::vocabulary::Digest([1; 32]);
pub const TWO_INSTS: limits::Limits = limits::Limits {
    mmdebstrap: hearth_protocol_mmdebstrap::limits::Limits { insts: 2, ..hearth_protocol_mmdebstrap::limits::SHIPPED },
    ..limits::SHIPPED
};
pub fn resolve(key: render::vocabulary::Digest, names: &[&[u8]]) -> render::vocabulary::Request {
    let packages = apt_packages();
    let mut sources = packages.sources;
    sources.architecture = bytes::copy_of(b"amd64");
    sources.components = Box::new([render::parts::packages::Component::Main]);
    sources.updates = false;
    sources.security = None;
    render::vocabulary::Request::Resolve {
        request: REQUEST,
        key,
        sources,
        names: resolve_names(names),
        variant: render::parts::packages::Variant::Apt,
        apt: packages.apt,
        deadline: skein_lib::Time::ZERO.saturating_add(Duration::from_secs(600)),
    }
}
pub fn made() -> file::Event {
    file::Event::Made { owner: crate::resolve::OWNER }
}
pub fn stored() -> file::Event {
    file::Event::Stored { owner: crate::resolve::OWNER, digest: io::digest::Digest([0; 4]) }
}
pub fn refused() -> file::Event {
    file::Event::Failed {
        owner: crate::resolve::OWNER,
        error: io::kernel::Error::Permission,
        committed: false,
        residue: None,
    }
}
impl Layer {
    pub fn resolution() -> Self {
        Self::resolution_with(limits::SHIPPED)
    }
    pub fn resolution_with(limits: limits::Limits) -> Self {
        let mut layer = Self::with_form(crate::Form::Text, limits);
        layer.protocol.store_path = bytes::copy_of(b"/scratch/home");
        layer
    }
    pub fn start_resolve(&mut self) {
        self.down(resolve(KEY, &[]));
        drop(self.asked());
        self.file_up(made());
        drop(self.asked());
        self.file_up(stored());
        let Some(Asked::Io(io::Request::Spawn { owner, .. })) = self.asked() else { panic!("spawn") };
        self.spawn_pending(owner, false);
        drop(self.asked());
        drop(self.asked());
    }
    pub fn advance_resolve(&mut self, span: Duration) {
        self.env.now = self.env.now.saturating_add(span);
        crate::fire(&mut self.protocol, &self.env, &mut self.above, &mut self.below);
    }
    pub fn deadline(&self) -> Option<skein_lib::Time> {
        crate::next_deadline(&self.protocol)
    }
    pub fn resolve_answers(&mut self, stdout: &[u8], stderr: &[u8], exit: Exit) -> Box<[builder::vocabulary::Event]> {
        let mut out = Held::new(bytes::copy_of(stdout));
        let mut err = Held::new(bytes::copy_of(stderr));
        let mut events = List::with_capacity(32);
        for _ in 0_u32..4096 {
            if let Some(event) = self.told() {
                events.push(event).expect("fixture events");
                continue;
            }
            let Some(request) = self.asked() else {
                if let Some(event) = self.told() {
                    events.push(event).expect("fixture events");
                }
                if self.protocol.resolving.is_none() {
                    return events.into_boxed();
                }
                panic!("resolve stalled");
            };
            match request {
                Asked::File(request) => self.resolve_file(request),
                Asked::Beneath(crate::Beneath::Remove { owner, .. }) => self.file_up(file::Event::Removed { owner }),
                Asked::Beneath(crate::Beneath::OpenDirectory { .. } | crate::Beneath::Rename { .. }) => {
                    panic!("empty fixture scratch")
                }
                Asked::Io(io::Request::Spawn { owner, .. }) => {
                    self.spawned(owner, false, exit);
                }
                Asked::Io(io::Request::Stream { stream, down: Down::Demand { read, .. } }) => {
                    if read != Read::Nothing {
                        let held = if stream == ERROR { &mut err } else { &mut out };
                        self.up(io::Event::Stream { owner: stream, up: held.answer(read).expect("read") });
                    }
                }
                Asked::Io(io::Request::Close { entity }) => {
                    let owner = if entity == Token::new(200) { self.owner.expect("owner") } else { entity };
                    self.up(io::Event::Closed { owner });
                }
                Asked::Io(_) => panic!("resolve operations"),
            }
        }
        panic!("bounded resolve settles");
    }
    pub fn resolve_file(&mut self, request: file::Request) {
        let event = match request {
            file::Request::MakeDirectory { .. } => made(),
            file::Request::Store { owner, .. } => file::Event::Stored { owner, digest: io::digest::Digest([0; 4]) },
            file::Request::Sync { owner, .. } | file::Request::SyncDirectory { owner, .. } => {
                file::Event::Synced { owner }
            }
            file::Request::OpenDirectory { owner, .. } => file::Event::Opened { owner, file: Token::new(88), len: 0 },
            file::Request::ListPage { owner, .. } => file::Event::Listed { owner, entries: Box::new([]) },
            file::Request::Close { owner, .. } => file::Event::Closed { owner },
            file::Request::Remove { owner, .. } => file::Event::Removed { owner },
            file::Request::Lock { .. }
            | file::Request::OpenPrivate { .. }
            | file::Request::Create { .. }
            | file::Request::CreateNoFollow { .. }
            | file::Request::OpenRead { .. }
            | file::Request::OpenReadNoFollow { .. }
            | file::Request::OpenWrite { .. }
            | file::Request::OpenWriteNoFollow { .. }
            | file::Request::Load { .. }
            | file::Request::Scan { .. }
            | file::Request::Stat { .. }
            | file::Request::WriteAt { .. }
            | file::Request::ReadAt { .. }
            | file::Request::Cut { .. }
            | file::Request::Rename { .. }
            | file::Request::List { .. } => panic!("resolve scratch operation"),
        };
        crate::up(&mut self.protocol, &self.env, Told::File(event), &mut self.above, &mut self.below);
    }
    pub fn resolve_exit(&mut self, exit: Exit) {
        self.up(io::Event::Exited { owner: crate::resolve::OWNER, exit });
    }
    pub fn resolve_line(&mut self, stream: Token, line: &[u8]) {
        self.up(io::Event::Stream { owner: stream, up: Up::Bytes(bytes::copy_of(line)) });
    }
    pub fn resolve_end(&mut self) {
        for owner in [OUTPUT, ERROR] {
            self.up(io::Event::Stream { owner, up: Up::End });
        }
    }
}
pub fn resolution_failure(events: &[builder::vocabulary::Event]) -> &render::vocabulary::Failure {
    let builder::vocabulary::Event::Render(render::vocabulary::Event::Failed { failure, .. }) = super::only(events)
    else {
        panic!("failed resolution")
    };
    failure
}
pub fn source_lines() -> Box<[vector::Line]> {
    Box::new([
        vector::Line {
            uri: bytes::copy_of(b"http://one/"),
            suite: bytes::copy_of(b"forky"),
            components: Box::new([vector::Component::Main]),
        },
        vector::Line {
            uri: bytes::copy_of(b"http://two/"),
            suite: bytes::copy_of(b"forky-updates"),
            components: Box::new([vector::Component::Main]),
        },
        vector::Line {
            uri: bytes::copy_of(b"http://three/"),
            suite: bytes::copy_of(b"forky-security"),
            components: Box::new([vector::Component::Main]),
        },
    ])
}

pub fn apt_packages() -> render::parts::packages::Packages {
    for packages in super::package_records() {
        if packages.variant == render::parts::packages::Variant::Apt {
            return packages;
        }
    }
    panic!("apt fixture")
}
fn resolve_names(names: &[&[u8]]) -> Box<[Box<[u8]>]> {
    let mut copied = List::with_capacity(u32::try_from(names.len()).expect("fixture names"));
    for name in names {
        copied.push(bytes::copy_of(name)).expect("name count");
    }
    copied.into_boxed()
}
pub fn mmdebstrap_name(bytes: &[u8]) -> vector::Name {
    vector::Name::new(bytes, &hearth_protocol_mmdebstrap::limits::SHIPPED).expect("fixture name")
}
#[must_use]
pub fn insts() -> [hearth_protocol_mmdebstrap::output::Inst; 2] {
    use hearth_protocol_mmdebstrap::output::Inst;
    [
        Inst {
            name: mmdebstrap_name(b"apt"),
            version: b"3.3.3".as_slice().into(),
            architecture: mmdebstrap_name(b"amd64"),
        },
        Inst {
            name: mmdebstrap_name(b"curl"),
            version: b"8.0".as_slice().into(),
            architecture: mmdebstrap_name(b"all"),
        },
    ]
}
#[must_use]
pub fn causes() -> [hearth_protocol_mmdebstrap::tail::Cause; 8] {
    use hearth_protocol_mmdebstrap::tail::Cause;
    [
        Cause::Fetch,
        Cause::Resolve,
        Cause::Space,
        Cause::Absent,
        Cause::Unknown(mmdebstrap_name(b"apt")),
        Cause::Unsatisfiable(Some(mmdebstrap_name(b"curl"))),
        Cause::Unsatisfiable(None),
        Cause::Other,
    ]
}

pub const CLOSURE_LIMIT: u32 = 8_192;

pub struct ResolvedRun {
    pub events: List<builder::vocabulary::Event>,
    pub trace: List<&'static str>,
}
impl Layer {
    pub fn resolve_with_indices(&mut self) -> ResolvedRun {
        self.indices_run(false)
    }
    pub fn index_arriving() -> (Self, ResolvedRun) {
        let mut layer = Self::resolution();
        layer.start_resolve();
        layer.resolve_end();
        layer.resolve_exit(Exit::Code(0));
        let run = layer.indices_run(true);
        (layer, run)
    }
    fn indices_run(&mut self, stop_at_index: bool) -> ResolvedRun {
        let mut run = ResolvedRun { events: List::with_capacity(8), trace: List::with_capacity(8) };
        let mut content = b"".as_slice();
        for _ in 0_u32..4096 {
            if let Some(event) = self.told() {
                if let builder::vocabulary::Event::Store(_) = &event {
                    continue;
                }
                if let builder::vocabulary::Event::Index { arrival, .. } = &event {
                    if stop_at_index {
                        run.events.push(event).expect("index");
                        return run;
                    }
                    self.client(builder::vocabulary::Request::Store(builder::store::vocabulary::Request::Discard {
                        arrival: *arrival,
                    }));
                    continue;
                }
                if let builder::vocabulary::Event::Render(render::vocabulary::Event::Resolved { .. }) = &event {
                    run.trace.push("resolved").expect("trace");
                }
                run.events.push(event).expect("events");
                continue;
            }
            let Some(request) = self.asked() else {
                if self.protocol.resolving.is_none() && self.above.is_empty() {
                    return run;
                }
                continue;
            };
            match request {
                Asked::File(file::Request::OpenDirectory { owner, name, .. }) => {
                    let lists = name.ends_with(hearth_protocol_mmdebstrap::lists::LISTS);
                    run.trace.push(if lists { "lists" } else { "scratch" }).expect("trace");
                    self.file_up(file::Event::Opened {
                        owner,
                        file: if lists { super::INDEX_DIRECTORY } else { Token::new(88) },
                        len: 0,
                    });
                }
                Asked::File(file::Request::ListPage { owner, directory, .. }) => {
                    let entries: Box<[file::Entry]> = if directory == super::INDEX_DIRECTORY {
                        super::index_entries(super::RESOLVE_INDEX_NAMES)
                    } else {
                        Box::new([])
                    };
                    self.file_up(file::Event::Listed { owner, entries });
                }
                Asked::File(file::Request::OpenReadNoFollow { owner, name, .. }) => {
                    content = if name.ends_with(b"_InRelease") {
                        run.trace.push("InRelease").expect("trace");
                        b"abc"
                    } else {
                        run.trace.push("Packages").expect("trace");
                        super::INDEX_PACKAGES
                    };
                    self.file_up(file::Event::Opened {
                        owner,
                        file: super::INDEX_FILE,
                        len: u64::try_from(content.len()).expect("fixture content"),
                    });
                }
                Asked::File(file::Request::ReadAt { owner, offset, max, .. }) => {
                    let start = usize::try_from(offset).expect("fixture offset");
                    let end = start.saturating_add(usize::try_from(max).expect("read bound")).min(content.len());
                    self.file_up(file::Event::Read {
                        owner,
                        bytes: bytes::copy_of(content.get(start..end).expect("read range")),
                    });
                }
                Asked::File(file::Request::Sync { owner, file }) => {
                    assert_eq!(file, super::INDEX_FILE);
                    if stop_at_index {
                        run.trace.push("sync").expect("trace");
                    }
                    self.file_up(file::Event::Synced { owner });
                }
                Asked::File(request) => {
                    if stop_at_index
                        && let file::Request::Close { file, .. } = &request
                        && *file == super::INDEX_FILE
                    {
                        run.trace.push("close").expect("trace");
                    }
                    self.resolve_file(request);
                }
                Asked::Io(io::Request::Close { entity }) => {
                    let owner = if entity == Token::new(200) { self.owner.expect("owner") } else { entity };
                    self.up(io::Event::Closed { owner });
                }
                Asked::Io(io::Request::Stream { .. }) => {}
                Asked::Io(_) | Asked::Beneath(_) => panic!("resolve index operations"),
            }
        }
        panic!("bounded resolution with indices settles");
    }
}
