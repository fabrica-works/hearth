//! The fake resolver answers typed install, stanza and index events.
use super::Run;
use hearth_domain_builder::vocabulary::Event;
use hearth_domain_render::{
    parts::packages::{Sources, Variant},
    resolution::{Costs, Index, Inst, Line, Stanza, Tools},
    vocabulary::Event as RenderEvent,
};
use hearth_fake_mmdebstrap::resolve::{self, Request};
use hearth_lib::encode::Digest;
use skein_lib::{Duration, Token};
impl Run {
    pub(super) fn resolve(
        &mut self,
        request: Token,
        sources: &Sources,
        names: &[Box<[u8]>],
        variant: Variant,
    ) -> Vec<Event> {
        let fake = self.shared.borrow();
        let universes = [&fake.universe];
        let variant = match variant {
            Variant::Apt => resolve::Variant::Apt,
            Variant::Container => resolve::Variant::Important,
        };
        let selected = resolve::closure(&Request { universes: &universes, variant, include: names })
            .expect("drawn universe resolves");
        let mut events = Vec::new();
        for selected in selected {
            let package = fake
                .universe
                .packages()
                .get(usize::try_from(selected.package).expect("package position fits"))
                .expect("selected package");
            events.push(Event::Render(RenderEvent::Inst {
                request,
                inst: Inst {
                    name: package.name.clone(),
                    version: package.version.clone(),
                    architecture: package.architecture.clone(),
                },
            }));
        }
        let bytes = fake.universe.index();
        let sha256 = Digest(skein_lib::sha256::digest(&bytes));
        for package in fake.universe.packages() {
            events.push(Event::Render(RenderEvent::Stanza {
                request,
                line: 0,
                stanza: Stanza {
                    package: package.name.clone(),
                    version: package.version.clone(),
                    architecture: package.architecture.clone(),
                    filename: package.filename.clone(),
                    size: package.size,
                    sha256: Digest(package.sha256),
                },
            }));
        }
        drop(fake);
        let arrival = self.store.arrival(self.files.as_mut().expect("world files"), sha256, &bytes);
        events.push(Event::Index { request, arrival, sha256, size: bytes.len() as u64, took: Duration::ZERO });
        events.push(Event::Render(RenderEvent::Resolved {
            request,
            lines: Box::new([Line {
                indices: Box::new([Index { path: b"Packages".as_slice().into(), size: bytes.len() as u64, sha256 }]),
                uri: b"https://snapshot.debian.org/archive/debian/20261001T030405Z/".as_slice().into(),
                suite: sources.suite.clone(),
                components: sources.components.iter().map(|component| component.name().into()).collect(),
            }]),
            tools: Tools { apt: b"3.1.8".as_slice().into(), mmdebstrap: b"1.5.7".as_slice().into() },
            costs: Costs { fetched: bytes.len() as u64, took: Duration::ZERO },
        }));
        events
    }
}
