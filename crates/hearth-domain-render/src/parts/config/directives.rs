use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Preset {
    pub unit: Box<[u8]>,
    pub action: Action,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Action {
    Enable,
    Disable,
}
impl Action {
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::Enable => b"enable",
            Self::Disable => b"disable",
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tmpfile {
    pub path: Box<[u8]>,
    pub line: Line,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Line {
    pub kind: LineKind,
    pub mode: Option<u16>,
    pub owner: Option<Box<[u8]>>,
    pub group: Option<Box<[u8]>>,
    pub age: Option<Box<[u8]>>,
    pub argument: Option<Box<[u8]>>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LineKind {
    D,
    CapitalD,
    E,
    F,
    FPlus,
    W,
    WPlus,
    CapitalL,
    CapitalLPlus,
    CapitalC,
    Z,
    CapitalZ,
    R,
    CapitalR,
}
impl LineKind {
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::D => b"d",
            Self::CapitalD => b"D",
            Self::E => b"e",
            Self::F => b"f",
            Self::FPlus => b"f+",
            Self::W => b"w",
            Self::WPlus => b"w+",
            Self::CapitalL => b"L",
            Self::CapitalLPlus => b"L+",
            Self::CapitalC => b"C",
            Self::Z => b"z",
            Self::CapitalZ => b"Z",
            Self::R => b"r",
            Self::CapitalR => b"R",
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Script {
    pub task: Box<[u8]>,
    pub order: u16,
    pub content_digest: hearth_lib::encode::Digest,
}

pub(super) fn encode(config: &super::Config, record: &mut hearth_lib::encode::Encoder) {
    record.key(b"presets");
    record.map(u64::try_from(config.presets.len()).expect("entry bound"));
    for preset in &config.presets {
        record.key(&preset.unit);
        record.bytes(preset.action.name());
    }
    encode_scripts(&config.scripts, record);
    encode_tmpfiles(&config.tmpfiles, record);
}
pub(crate) fn encode_scripts(scripts: &[Script], record: &mut hearth_lib::encode::Encoder) {
    record.key(b"scripts");
    record.list(u64::try_from(scripts.len()).expect("task bound"));
    for script in scripts {
        record.map(3);
        record.key(b"content_digest");
        record.digest(&script.content_digest);
        record.key(b"order");
        record.uint(u64::from(script.order));
        record.key(b"task");
        record.bytes(&script.task);
    }
}
fn encode_tmpfiles(tmpfiles: &[Tmpfile], record: &mut hearth_lib::encode::Encoder) {
    record.key(b"tmpfiles");
    record.map(u64::try_from(tmpfiles.len()).expect("entry bound"));
    for tmpfile in tmpfiles {
        record.key(&tmpfile.path);
        let line = &tmpfile.line;
        let mut fields = 1_u64;
        for present in [
            line.mode.is_some(),
            line.owner.is_some(),
            line.group.is_some(),
            line.age.is_some(),
            line.argument.is_some(),
        ] {
            fields = fields.saturating_add(u64::from(present));
        }
        record.map(fields);
        optional(record, b"age", line.age.as_deref());
        optional(record, b"argument", line.argument.as_deref());
        optional(record, b"group", line.group.as_deref());
        if let Some(mode) = line.mode {
            record.key(b"mode");
            record.uint(u64::from(mode));
        }
        optional(record, b"owner", line.owner.as_deref());
        record.key(b"type");
        record.bytes(line.kind.name());
    }
}
fn optional(record: &mut hearth_lib::encode::Encoder, key: &[u8], value: Option<&[u8]>) {
    if let Some(value) = value {
        record.key(key);
        record.bytes(value);
    }
}
