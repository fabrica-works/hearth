use super::{Box, Digest, FIELD_AT, List, Name, Plan, copy_of};
use crate::{
    entries::Expansion,
    evaluate::{Missing, template::Template},
    limits::SHIPPED,
    revision::{
        Entry, Mode, Oid,
        declaration::{Declaration, Literal, Map},
    },
};
pub const NAMED_TEMPLATE: &[u8] = b"tasks/motd/templates/greeting.j2";
pub fn source_entry(name: Name, source: &[u8]) -> crate::revision::declaration::Entry {
    let mut entry = super::super::entry(super::super::plain(b"/etc/greeting"), None, None);
    entry.fields = Box::new([super::super::field(name, Literal::Plain { at: FIELD_AT, text: copy_of(source) })]);
    entry
}
pub fn source_declaration(entry: crate::revision::declaration::Entry) -> Declaration {
    super::super::entry_maps([super::super::entry_map(Map::Files, [entry])])
}
pub fn source_listing(path: &[u8], mode: Mode, size: u64) -> Entry {
    Entry { path: copy_of(path), oid: Oid(copy_of(path)), mode, size }
}
pub fn sources_checked() -> crate::schema::Validated {
    let roots = super::super::Roots::x1_prod();
    crate::schema::Validated {
        tasks: Box::new([crate::schema::Checked {
            name: copy_of(b"motd"),
            settings: roots.settings,
            tasks: roots.tasks,
        }]),
    }
}
pub fn source_side(declared: &Declaration, listing: &[Entry]) -> crate::call::Side {
    let mut side = crate::call::Side::new(copy_of(b"repository"), Box::new([]), &SHIPPED);
    side.made = skein_lib::Map::with_capacity(SHIPPED.targets);
    side.entries = listing.into();
    side.commit = Some(Oid(copy_of(b"commit")));
    side.documents = super::super::declarations(&[(b"motd", Some(declared.clone()))]);
    let capacity = u32::try_from(listing.len()).expect("fixture count");
    side.cached = skein_lib::Map::with_capacity(capacity);
    side.templates = skein_lib::Map::with_capacity(capacity);
    side.sizes = skein_lib::Map::with_capacity(SHIPPED.expansion);
    for entry in listing {
        side.cached.insert(entry.oid.clone(), Digest([1; 32])).expect("fixture count");
        side.templates
            .insert(entry.oid.clone(), super::super::template(&[0], [super::super::text(0, b"hello")]))
            .expect("fixture count");
    }
    side
}
pub fn source_expansion(declared: &Declaration) -> Expansion {
    let mut expanded = super::super::expansion(declared, &SHIPPED, SHIPPED.expanded).expect("expanded inputs");
    expanded.task = copy_of(b"motd");
    Expansion { tasks: Box::new([expanded]) }
}
pub fn planned_sources(declared: &Declaration, listing: &[Entry]) -> Result<Box<[Plan]>, Box<[crate::errors::Error]>> {
    let side = source_side(declared, listing);
    let mut errors = List::with_capacity(SHIPPED.expansion);
    let plans = side.plans(&super::super::meta(), &sources_checked(), &source_expansion(declared), &mut errors);
    plans.ok_or(errors.into_boxed())
}
pub fn element_declaration() -> Declaration {
    element_source(Name::Template, b"templates/greeting.j2")
}
fn element_source(name: Name, source: &[u8]) -> Declaration {
    let mut entry = super::super::list_entry();
    entry.fields = source_entry(name, source).fields;
    source_declaration(entry)
}
pub fn element_template() -> Template {
    super::super::template(
        &[0],
        [
            super::super::output(0, super::super::expr(1, [super::super::name(3, b"key", Missing::Error)])),
            super::super::text(10, b":"),
            super::super::output(11, super::super::expr(1, [super::super::name(14, b"value", Missing::Error)])),
        ],
    )
}
pub type SourceCall =
    (crate::Render, skein_lib::Env<crate::limits::Limits>, skein_lib::Queue<crate::vocabulary::Output>);
pub fn source_render(declared: &Declaration, listing: &[Entry], template: Option<Template>) -> SourceCall {
    let mut side = source_side(declared, listing);
    if let Some(template) = template {
        side.templates.insert(Oid(copy_of(NAMED_TEMPLATE)), template).expect("fixture count");
    }
    side.cached = skein_lib::Map::with_capacity(u32::try_from(listing.len()).expect("fixture count"));
    let plans = planned_sources(declared, listing).expect("valid sources");
    side.rendering = Some(crate::call::templates::Target::new(
        super::super::meta(),
        Box::new([]),
        Digest([1; 32]),
        sources_checked(),
        super::super::Roots::x1_prod().debian,
        super::super::template_prepared(plans, source_expansion(declared)),
        SHIPPED.expansion,
    ));
    side.ready = true;
    side.files = true;
    let render = crate::Render { call: Some(crate::call::Call { side, diff: None }) };
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: SHIPPED };
    (render, env, skein_lib::Queue::with_capacity(1))
}
pub type Kept = (Box<[u8]>, Box<[u8]>);
pub fn source_keeps(declared: &Declaration, template: Template) -> Box<[Kept]> {
    use crate::vocabulary::{Event, Output, Request};
    let (mut render, env, mut out) =
        source_render(declared, &[source_listing(NAMED_TEMPLATE, Mode::File, 20)], Some(template));
    let mut keeps = List::with_capacity(SHIPPED.expansion);
    for _ in 0..SHIPPED.expansion {
        if !render.work_pending() {
            break;
        }
        crate::resume(&mut render, &env, &mut out);
        match out.pop() {
            Some(Output::Request(Request::Keep { request, sha256, path, bytes })) => {
                keeps.push((path, bytes)).expect("expansion bound");
                crate::step(&mut render, &env, Event::Kept { request, sha256 }, &mut out);
            }
            Some(Output::Request(Request::Find { request, .. })) => {
                crate::step(&mut render, &env, Event::Absent { request }, &mut out);
            }
            Some(Output::Answer(_) | Output::Request(_)) | None => {}
        }
    }
    keeps.into_boxed()
}

pub fn dropped_template_declaration() -> Declaration {
    let mut entry = source_entry(Name::Template, b"templates/greeting.j2");
    entry.when = Some(super::super::condition([super::super::op(0, crate::evaluate::Op::Boolean(false))]));
    source_declaration(entry)
}
pub fn copy_declaration() -> Declaration {
    source_declaration(source_entry(Name::Copy, b"lab-ca/lab.crt"))
}
pub fn copy_render() -> SourceCall {
    source_render(&element_source(Name::Copy, b"lab-ca/lab.crt"), &super::super::forms::listing(Mode::File, 20), None)
}
