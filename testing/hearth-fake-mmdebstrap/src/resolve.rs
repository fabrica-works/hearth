//! Mark requests before dependencies, using the model of apt in decisions.md, 14.
use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    vec::Vec,
};
use core::{cell::RefCell, cmp::Ordering};
use hearth_fake_machine::hearth::{
    universe::{Alternative, Package, Priority, Universe, alternative_text},
    version,
};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Variant {
    Essential,
    Apt,
    Required,
    Important,
    Standard,
}
#[derive(Debug)]
pub struct Request<'a> {
    pub universes: &'a [&'a Universe],
    pub variant: Variant,
    pub include: &'a [Box<[u8]>],
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Selected {
    pub universe: u32,
    pub package: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Field {
    PreDepends,
    Depends,
    Conflicts,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Broken {
    pub package: Box<[u8]>,
    pub field: Field,
    pub relation: Box<[u8]>,
    pub against: Option<Box<[u8]>>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Failure {
    Unknown(Box<[u8]>),
    NoCandidate(Box<[u8]>),
    Unsatisfiable(Box<[Broken]>),
}
pub const MOST_BROKEN: u32 = 64;
struct Resolver<'a> {
    packages: Vec<(Selected, &'a Package)>,
    candidates: BTreeMap<&'a [u8], usize>,
    marked: Vec<usize>,
    read: Option<RefCell<BTreeSet<usize>>>,
}
impl<'a> Resolver<'a> {
    fn new(universes: &[&'a Universe]) -> Self {
        let mut packages = Vec::new();
        let mut candidates = BTreeMap::new();
        for (universe, source) in universes.iter().enumerate() {
            for (package, value) in source.packages().iter().enumerate() {
                let selected = Selected {
                    universe: u32::try_from(universe).expect("source count fits u32"),
                    package: u32::try_from(package).expect("package count fits u32"),
                };
                let place = packages.len();
                packages.push((selected, value));
                let previous = candidates.entry(value.name.as_ref()).or_insert(place);
                if version::compare(&value.version, &packages[*previous].1.version) == Ordering::Greater {
                    *previous = place;
                }
            }
        }
        Self { packages, candidates, marked: Vec::new(), read: None }
    }
    fn read_name(&self, name: &[u8]) {
        let Some(read) = &self.read else {
            return;
        };
        read.borrow_mut().extend(
            self.packages.iter().enumerate().filter(|(_, (_, package))| package.name.as_ref() == name).map(|(i, _)| i),
        );
    }
    fn mark(&mut self, place: usize) {
        self.read_name(&self.packages[place].1.name);
        if !self.marked.contains(&place) {
            self.marked.push(place);
        }
    }
    fn providers(&self, alternative: &Alternative) -> Vec<usize> {
        let mut found: Vec<_> = self
            .packages
            .iter()
            .enumerate()
            .filter(|(i, (_, p))| self.candidates.get(p.name.as_ref()) == Some(i) && provides(p, alternative))
            .map(|(i, _)| i)
            .collect();
        for place in &found {
            self.read_name(&self.packages[*place].1.name);
        }
        found.sort_by_key(|i| (!self.packages[*i].1.essential, self.packages[*i].1.priority, *i));
        found
    }
    fn request(&mut self, name: &[u8]) -> Result<(), Failure> {
        if let Some(place) = self.candidates.get(name) {
            self.mark(*place);
            return Ok(());
        }
        let alternative = Alternative { name: name.into(), qualifier: None, version: None };
        let providers = self.providers(&alternative);
        match providers.as_slice() {
            [] => Err(Failure::Unknown(name.into())),
            [place] => {
                self.mark(*place);
                Ok(())
            }
            [_, _, ..] => Err(Failure::NoCandidate(name.into())),
        }
    }
    fn free(&self, place: usize) -> bool {
        let package = self.packages[place].1;
        self.marked.iter().all(|i| {
            *i == place || {
                let other = self.packages[*i].1;
                !package.conflicts.iter().any(|a| conflicts(other, a))
                    && !other.conflicts.iter().any(|a| conflicts(package, a))
            }
        })
    }
    fn meet(&mut self, group: &[Alternative]) -> bool {
        if self.marked.iter().any(|i| group.iter().any(|a| satisfies(self.packages[*i].1, a))) {
            return true;
        }
        let mut fallback = None;
        for alternative in group {
            self.read_name(&alternative.name);
            let direct = self
                .candidates
                .get(alternative.name.as_ref())
                .copied()
                .filter(|i| satisfies(self.packages[*i].1, alternative));
            if fallback.is_none() {
                fallback = direct;
            }
            if let Some(place) = direct.filter(|i| self.free(*i)) {
                self.mark(place);
                return true;
            }
            if let Some(place) = self.providers(alternative).into_iter().find(|i| self.free(*i)) {
                self.mark(place);
                return true;
            }
        }
        if let Some(place) = fallback {
            self.mark(place);
            return true;
        }
        false
    }
}
fn version_holds(version: &[u8], alternative: &Alternative) -> bool {
    alternative.version.as_ref().is_none_or(|(relation, wanted)| relation.holds(version, wanted))
}
fn provides(package: &Package, alternative: &Alternative) -> bool {
    package.provides.iter().any(|p| {
        p.name == alternative.name
            && alternative.version.as_ref().is_none_or(|(relation, wanted)| {
                p.version.as_ref().is_some_and(|version| relation.holds(version, wanted))
            })
    })
}
fn satisfies(package: &Package, alternative: &Alternative) -> bool {
    (package.name == alternative.name && version_holds(&package.version, alternative)) || provides(package, alternative)
}
fn conflicts(package: &Package, alternative: &Alternative) -> bool {
    (package.name == alternative.name && version_holds(&package.version, alternative))
        || (alternative.version.is_none() && provides(package, alternative))
}
fn broken(package: &Package, field: Field, group: &[Alternative], against: Option<Box<[u8]>>) -> Broken {
    let mut relation = Vec::new();
    for (i, alternative) in group.iter().enumerate() {
        if i != 0 {
            relation.extend_from_slice(b" | ");
        }
        relation.extend_from_slice(&alternative_text(alternative));
    }
    Broken { package: package.name.clone(), field, relation: relation.into_boxed_slice(), against }
}
/// Resolve a variant and its explicit requests, returning the closure in marking order.
pub fn closure(request: &Request<'_>) -> Result<Box<[Selected]>, Failure> {
    resolve(&mut Resolver::new(request.universes), request)
}
/// Packages marked, candidates weighed and providers ranked, in universe order.
#[must_use]
pub fn read_by(request: &Request<'_>) -> Box<[Selected]> {
    let mut resolver = Resolver::new(request.universes);
    resolver.read = Some(RefCell::new(BTreeSet::new()));
    let _ = resolve(&mut resolver, request);
    resolver.read.as_ref().expect("read collection enabled").borrow().iter().map(|i| resolver.packages[*i].0).collect()
}
fn resolve(resolver: &mut Resolver<'_>, request: &Request<'_>) -> Result<Box<[Selected]>, Failure> {
    for i in 0..resolver.packages.len() {
        let package = resolver.packages[i].1;
        if package.essential {
            resolver.request(&package.name)?;
        }
    }
    if request.variant != Variant::Essential {
        resolver.request(b"apt")?;
    }
    let priority = match request.variant {
        Variant::Essential | Variant::Apt => None,
        Variant::Required => Some(Priority::Required),
        Variant::Important => Some(Priority::Important),
        Variant::Standard => Some(Priority::Standard),
    };
    for i in 0..resolver.packages.len() {
        let package = resolver.packages[i].1;
        if priority.is_some_and(|p| package.priority <= p) {
            resolver.request(&package.name)?;
        }
    }
    for name in request.include {
        resolver.request(name)?;
    }
    // Each candidate can be marked once, so this worklist is bounded by the input package count.
    for position in 0..resolver.packages.len() {
        let Some(place) = resolver.marked.get(position).copied() else {
            break;
        };
        let package = resolver.packages[place].1;
        for group in package.pre_depends.iter().chain(package.depends.iter()) {
            resolver.meet(group);
        }
    }
    let mut failures = Vec::new();
    for place in &resolver.marked {
        let package = resolver.packages[*place].1;
        for (field, groups) in [(Field::PreDepends, &package.pre_depends), (Field::Depends, &package.depends)] {
            for group in groups {
                if failures.len() < usize::try_from(MOST_BROKEN).expect("bound fits usize")
                    && !resolver.marked.iter().any(|i| group.iter().any(|a| satisfies(resolver.packages[*i].1, a)))
                {
                    failures.push(broken(package, field, group, None));
                }
            }
        }
        for alternative in &package.conflicts {
            for other in &resolver.marked {
                if failures.len() < usize::try_from(MOST_BROKEN).expect("bound fits usize")
                    && place != other
                    && conflicts(resolver.packages[*other].1, alternative)
                {
                    failures.push(broken(
                        package,
                        Field::Conflicts,
                        core::slice::from_ref(alternative),
                        Some(resolver.packages[*other].1.version.clone()),
                    ));
                }
            }
        }
    }
    if failures.is_empty() {
        Ok(resolver.marked.iter().map(|i| resolver.packages[*i].0).collect())
    } else {
        Err(Failure::Unsatisfiable(failures.into_boxed_slice()))
    }
}
