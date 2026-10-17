//! The M1 package index (testing.md, 3.6), with Policy 7.1 relations.
use super::version;
use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    format,
    vec::Vec,
};
use core::cmp::Ordering;
pub const KEPT: [&str; 14] = [
    "Package",
    "Version",
    "Essential",
    "Installed-Size",
    "Architecture",
    "Provides",
    "Depends",
    "Pre-Depends",
    "Conflicts",
    "Section",
    "Priority",
    "Filename",
    "Size",
    "SHA256",
];
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Priority {
    Required,
    Important,
    Standard,
    Optional,
    Extra,
}
impl Priority {
    #[must_use]
    pub const fn text(self) -> &'static [u8] {
        match self {
            Self::Required => b"required",
            Self::Important => b"important",
            Self::Standard => b"standard",
            Self::Optional => b"optional",
            Self::Extra => b"extra",
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Relation {
    Earlier,
    EarlierOrEqual,
    Equal,
    LaterOrEqual,
    Later,
}
impl Relation {
    #[must_use]
    pub fn holds(self, have: &[u8], wanted: &[u8]) -> bool {
        let order = version::compare(have, wanted);
        match self {
            Self::Earlier => order == Ordering::Less,
            Self::EarlierOrEqual => order != Ordering::Greater,
            Self::Equal => order == Ordering::Equal,
            Self::LaterOrEqual => order != Ordering::Less,
            Self::Later => order == Ordering::Greater,
        }
    }
    #[must_use]
    pub const fn text(self) -> &'static [u8] {
        match self {
            Self::Earlier => b"<<",
            Self::EarlierOrEqual => b"<=",
            Self::Equal => b"=",
            Self::LaterOrEqual => b">=",
            Self::Later => b">>",
        }
    }
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Alternative {
    pub name: Box<[u8]>,
    pub qualifier: Option<Box<[u8]>>,
    pub version: Option<(Relation, Box<[u8]>)>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Provide {
    pub name: Box<[u8]>,
    pub version: Option<Box<[u8]>>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Package {
    pub name: Box<[u8]>,
    pub version: Box<[u8]>,
    pub architecture: Box<[u8]>,
    pub priority: Priority,
    pub essential: bool,
    pub section: Box<[u8]>,
    pub installed_size: Option<u64>,
    pub provides: Box<[Provide]>,
    pub depends: Box<[Box<[Alternative]>]>,
    pub pre_depends: Box<[Box<[Alternative]>]>,
    pub conflicts: Box<[Alternative]>,
    pub filename: Box<[u8]>,
    pub size: u64,
    pub sha256: [u8; 32],
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Universe {
    packages: Box<[Package]>,
    names: BTreeMap<Box<[u8]>, u32>,
    providers: BTreeMap<Box<[u8]>, Vec<u32>>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct IndexError {
    pub line: u32,
    pub wrong: Wrong,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Wrong {
    Missing(&'static str),
    Twice(&'static str),
    Form(&'static str),
    Duplicate,
}
impl Universe {
    #[must_use]
    pub fn new(packages: Box<[Package]>) -> Self {
        let mut names = BTreeMap::new();
        let mut providers: BTreeMap<Box<[u8]>, Vec<u32>> = BTreeMap::new();
        for (place, package) in packages.iter().enumerate() {
            let place = u32::try_from(place).expect("a universe fits u32");
            assert!(names.insert(package.name.clone(), place).is_none(), "one package a name");
            for provide in &package.provides {
                providers.entry(provide.name.clone()).or_default().push(place);
            }
        }
        Self { packages, names, providers }
    }
    pub fn read(index: &[u8]) -> Result<Self, IndexError> {
        let mut packages = Vec::new();
        let mut fields = BTreeMap::new();
        let mut start = 1;
        let mut names = BTreeSet::new();
        let mut active = false;
        for (place, line) in index.split(|b| *b == b'\n').chain(core::iter::once(b"".as_slice())).enumerate() {
            let number = u32::try_from(place).expect("index fits u32").checked_add(1).expect("line fits u32");
            if line.is_empty() {
                if active {
                    let package = read_package(&fields, start)?;
                    if !names.insert(package.name.clone()) {
                        return Err(IndexError { line: start, wrong: Wrong::Duplicate });
                    }
                    packages.push(package);
                    fields.clear();
                    active = false;
                }
                start = number.checked_add(1).expect("line fits u32");
                continue;
            }
            if line.first().is_some_and(u8::is_ascii_whitespace) {
                continue;
            }
            active = true;
            let Some((name, value)) = version::split(line, b':', false) else {
                return Err(IndexError { line: number, wrong: Wrong::Form("field") });
            };
            if let Some(name) = KEPT.iter().find(|kept| kept.as_bytes().eq_ignore_ascii_case(name))
                && fields.insert(*name, (value.trim_ascii(), number)).is_some()
            {
                return Err(IndexError { line: number, wrong: Wrong::Twice(name) });
            }
        }
        Ok(Self::new(packages.into_boxed_slice()))
    }
    #[must_use]
    pub fn packages(&self) -> &[Package] {
        &self.packages
    }
    #[must_use]
    pub fn package(&self, name: &[u8]) -> Option<&Package> {
        self.names.get(name).and_then(|place| self.packages.get(usize::try_from(*place).expect("place fits usize")))
    }
    #[must_use]
    pub fn providers(&self, name: &[u8]) -> &[u32] {
        self.providers.get(name).map_or(&[], Vec::as_slice)
    }
    #[must_use]
    pub fn index(&self) -> Box<[u8]> {
        let mut out = Vec::new();
        for p in &self.packages {
            field(&mut out, "Package", &p.name);
            field(&mut out, "Version", &p.version);
            if p.essential {
                field(&mut out, "Essential", b"yes");
            }
            if let Some(size) = p.installed_size {
                field(&mut out, "Installed-Size", format!("{size}").as_bytes());
            }
            field(&mut out, "Architecture", &p.architecture);
            let provides: Vec<Box<[Alternative]>> = p
                .provides
                .iter()
                .map(|p| {
                    Box::from([Alternative {
                        name: p.name.clone(),
                        qualifier: None,
                        version: p.version.clone().map(|v| (Relation::Equal, v)),
                    }])
                })
                .collect();
            relations_field(&mut out, "Provides", &provides);
            relations_field(&mut out, "Depends", &p.depends);
            relations_field(&mut out, "Pre-Depends", &p.pre_depends);
            let conflicts: Vec<Box<[Alternative]>> = p.conflicts.iter().map(|a| Box::from([a.clone()])).collect();
            relations_field(&mut out, "Conflicts", &conflicts);
            field(&mut out, "Section", &p.section);
            field(&mut out, "Priority", p.priority.text());
            field(&mut out, "Filename", &p.filename);
            field(&mut out, "Size", format!("{}", p.size).as_bytes());
            let mut sha = Vec::new();
            for byte in p.sha256 {
                for nibble in [byte / 16, byte % 16] {
                    sha.push(*b"0123456789abcdef".get(usize::from(nibble)).expect("a nibble"));
                }
            }
            field(&mut out, "SHA256", &sha);
            out.push(b'\n');
        }
        out.into_boxed_slice()
    }
}
fn field(out: &mut Vec<u8>, name: &str, value: &[u8]) {
    out.extend_from_slice(name.as_bytes());
    out.extend_from_slice(b": ");
    out.extend_from_slice(value);
    out.push(b'\n');
}
fn relations_field(out: &mut Vec<u8>, name: &str, groups: &[Box<[Alternative]>]) {
    if groups.is_empty() {
        return;
    }
    let mut value = Vec::new();
    for (i, group) in groups.iter().enumerate() {
        if i != 0 {
            value.extend_from_slice(b", ");
        }
        for (j, a) in group.iter().enumerate() {
            if j != 0 {
                value.extend_from_slice(b" | ");
            }
            value.extend_from_slice(&alternative_text(a));
        }
    }
    field(out, name, &value);
}
#[must_use]
pub fn alternative_text(a: &Alternative) -> Box<[u8]> {
    let mut value = a.name.to_vec();
    if let Some(q) = &a.qualifier {
        value.push(b':');
        value.extend_from_slice(q);
    }
    if let Some((r, v)) = &a.version {
        value.extend_from_slice(b" (");
        value.extend_from_slice(r.text());
        value.push(b' ');
        value.extend_from_slice(v);
        value.push(b')');
    }
    value.into_boxed_slice()
}
type Fields<'a> = BTreeMap<&'static str, (&'a [u8], u32)>;
fn read_package(fields: &Fields<'_>, start: u32) -> Result<Package, IndexError> {
    let get = |name| fields.get(name).copied().ok_or(IndexError { line: start, wrong: Wrong::Missing(name) });
    let checked = |name, check: fn(&[u8]) -> bool| {
        let (v, line) = get(name)?;
        if check(v) { Ok(Box::<[u8]>::from(v)) } else { Err(IndexError { line, wrong: Wrong::Form(name) }) }
    };
    let optional = |name| fields.get(name).map(|(v, line)| (*v, *line));
    let decimal = |name| {
        let (v, line) = get(name)?;
        decimal(v).ok_or(IndexError { line, wrong: Wrong::Form(name) })
    };
    let groups = |name| {
        optional(name).map_or_else(
            || Ok(Box::default()),
            |(v, line)| relations(v).ok_or(IndexError { line, wrong: Wrong::Form(name) }),
        )
    };
    let (priority, line) = get("Priority")?;
    let priority = match priority {
        b"required" => Priority::Required,
        b"important" => Priority::Important,
        b"standard" => Priority::Standard,
        b"optional" => Priority::Optional,
        b"extra" => Priority::Extra,
        _ => return Err(IndexError { line, wrong: Wrong::Form("Priority") }),
    };
    let essential = match optional("Essential") {
        None => false,
        Some((b"yes", _)) => true,
        Some((_, line)) => return Err(IndexError { line, wrong: Wrong::Form("Essential") }),
    };
    let installed_size = optional("Installed-Size")
        .map(|(v, line)| self::decimal(v).ok_or(IndexError { line, wrong: Wrong::Form("Installed-Size") }))
        .transpose()?;
    let mut provides = Vec::new();
    for group in groups("Provides")? {
        if group.len() != 1 {
            return Err(IndexError { line: get("Provides")?.1, wrong: Wrong::Form("Provides") });
        }
        let a = group.first().expect("one provided name");
        if a.qualifier.is_some() || a.version.as_ref().is_some_and(|(r, _)| *r != Relation::Equal) {
            return Err(IndexError { line: get("Provides")?.1, wrong: Wrong::Form("Provides") });
        }
        provides.push(Provide { name: a.name.clone(), version: a.version.as_ref().map(|(_, v)| v.clone()) });
    }
    let mut conflicts = Vec::new();
    for group in groups("Conflicts")? {
        if group.len() != 1 {
            return Err(IndexError { line: get("Conflicts")?.1, wrong: Wrong::Form("Conflicts") });
        }
        conflicts.extend(group);
    }
    let (sha, line) = get("SHA256")?;
    let sha256 = hex(sha).ok_or(IndexError { line, wrong: Wrong::Form("SHA256") })?;
    Ok(Package {
        name: checked("Package", name)?,
        version: checked("Version", valid_version)?,
        architecture: checked("Architecture", word)?,
        priority,
        essential,
        section: checked("Section", path)?,
        installed_size,
        provides: provides.into_boxed_slice(),
        depends: groups("Depends")?,
        pre_depends: groups("Pre-Depends")?,
        conflicts: conflicts.into_boxed_slice(),
        filename: checked("Filename", path)?,
        size: decimal("Size")?,
        sha256,
    })
}
fn decimal(v: &[u8]) -> Option<u64> {
    if v.is_empty() {
        return None;
    }
    v.iter().try_fold(0_u64, |n, b| {
        if b.is_ascii_digit() { n.checked_mul(10)?.checked_add(u64::from(b.checked_sub(b'0')?)) } else { None }
    })
}
fn word(v: &[u8]) -> bool {
    !v.is_empty() && v.iter().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
}
fn name(v: &[u8]) -> bool {
    v.len() >= 2
        && v.first().is_some_and(u8::is_ascii_alphanumeric)
        && v.iter().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"+-.".contains(b))
}
fn path(v: &[u8]) -> bool {
    !v.is_empty() && v.iter().all(u8::is_ascii_graphic)
}
fn valid_version(v: &[u8]) -> bool {
    let (epoch, rest) = version::split(v, b':', false).map_or((None, v), |(a, b)| (Some(a), b));
    if epoch.is_some_and(|e| e.is_empty() || !e.iter().all(u8::is_ascii_digit)) {
        return false;
    }
    let (upstream, revision) = version::split(rest, b'-', true).map_or((rest, None), |(a, b)| (a, Some(b)));
    upstream.first().is_some_and(u8::is_ascii_digit)
        && upstream.iter().all(|b| b.is_ascii_alphanumeric() || b".+-~".contains(b))
        && revision.is_none_or(|r| !r.is_empty() && r.iter().all(|b| b.is_ascii_alphanumeric() || b"+.~".contains(b)))
}
fn hex(v: &[u8]) -> Option<[u8; 32]> {
    if v.len() != 64 {
        return None;
    }
    let mut out = [0; 32];
    for (slot, pair) in out.iter_mut().zip(v.chunks_exact(2)) {
        let mut n = 0_u8;
        for b in pair {
            let d = char::from(*b).to_digit(16)?;
            n = n.checked_mul(16)?.checked_add(u8::try_from(d).ok()?)?;
        }
        *slot = n;
    }
    Some(out)
}
fn relations(v: &[u8]) -> Option<Box<[Box<[Alternative]>]>> {
    v.split(|b| *b == b',').map(|g| g.split(|b| *b == b'|').map(alternative).collect::<Option<Box<[_]>>>()).collect()
}
fn alternative(v: &[u8]) -> Option<Alternative> {
    let v = v.trim_ascii();
    let end = v.iter().position(|b| b.is_ascii_whitespace() || *b == b'(').unwrap_or(v.len());
    let (head, tail) = v.split_at_checked(end)?;
    let (n, qualifier) = version::split(head, b':', false).map_or((head, None), |(n, q)| (n, Some(q)));
    if !name(n) || qualifier.is_some_and(|q| !word(q)) {
        return None;
    }
    let tail = tail.trim_ascii();
    let version = if tail.is_empty() {
        None
    } else {
        let inner = tail.strip_prefix(b"(")?.strip_suffix(b")")?.trim_ascii();
        let length = inner.iter().take_while(|b| b"<=>".contains(b)).count();
        let (op, wanted) = inner.split_at_checked(length)?;
        let wanted = wanted.trim_ascii();
        let r = match op {
            b"<<" => Relation::Earlier,
            b"<=" => Relation::EarlierOrEqual,
            b"=" => Relation::Equal,
            b">=" => Relation::LaterOrEqual,
            b">>" => Relation::Later,
            _ => return None,
        };
        if !valid_version(wanted) {
            return None;
        }
        Some((r, wanted.into()))
    };
    Some(Alternative { name: n.into(), qualifier: qualifier.map(Box::from), version })
}
