use super::{Repository, entry, oid, with_task};
use crate::parent;
use hearth_domain_render::{
    self as render,
    limits::{self, SHIPPED},
    vocabulary::{As, Event, Output, Request},
};
use skein_lib::{Env, Queue, Time, Wall};
use skein_world::domain::heap::Meter;

// A one-MiB newline-only template holds its text and 1,048,577 u32 offsets.
// The decoded charge adds the one instruction's 32 bytes (render.md, 2.3).
pub const DECODED_TEMPLATE_BYTES: u64 = 5_242_916; // Text + 32 + 1,048,577 * 4.
pub const DECODED_TEMPLATES: u32 = 12; // 67,108,864 / 5,242,916.
// Sixteen one-MiB comments have 1,048,573 lines each and almost fill 64 MiB.
pub const COMMENT_TEMPLATES: u32 = 16;
#[must_use]
pub fn newline_template() -> hearth_domain_render::evaluate::template::Template {
    use hearth_domain_render::evaluate::template::{Does, Instruction, Template};
    let bytes = usize::try_from(SHIPPED.template).expect("template bound");
    let text = vec![b'\n'; bytes];
    Template {
        instructions: Box::new([Instruction { at: 0, does: Does::Text(text.into()) }]),
        lines: (0..=SHIPPED.template).collect(),
    }
}

const DECODE_OUTPUTS: u32 = 1; // The domain emits one request at a time.
const DECODE_STEPS: u32 = 16; // Two read phases and the resumes between them.
#[must_use]
pub fn decoded_templates_repository() -> Repository {
    templates_repository(DECODED_TEMPLATES)
}
#[must_use]
pub fn decoded_comments_repository() -> Repository {
    templates_repository(COMMENT_TEMPLATES)
}
fn templates_repository(count: u32) -> Repository {
    let mut repository = with_task();
    for index in 0..count {
        let mut file =
            entry(format!("tasks/motd/tree/file-{index}.j2").as_bytes(), u8::try_from(index).expect("template id"));
        file.size = u64::from(SHIPPED.template);
        repository.entries.push(file);
    }
    repository.entries.sort_by(|first, second| first.path.cmp(&second.path));
    repository
}
#[must_use]
pub fn comment_template() -> render::evaluate::template::Template {
    // {#, then template - 4 newlines, then #}: every first-byte offset stays.
    let newlines = SHIPPED.template - 4;
    let mut lines = Vec::with_capacity((newlines + 1) as usize);
    lines.push(0);
    lines.extend(3..=newlines + 2);
    render::evaluate::template::Template { instructions: Box::new([]), lines: lines.into() }
}
#[must_use]
pub fn decoded_bound() -> u64 {
    render::revision::decoded_worst_case(&SHIPPED).expect("decoded bound")
}
pub fn checked_decoding(repository: &Repository, bound: u64, template: &render::evaluate::template::Template) {
    let meter = Meter::new();
    let mut domain = render::Render::new(&SHIPPED);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: SHIPPED };
    let mut output = Queue::with_capacity(DECODE_OUTPUTS);
    decoded_step(
        &meter,
        bound,
        &mut domain,
        &env,
        &mut output,
        Some(Event::Render {
            resolve: false,
            repository: Box::from(b"/repo".as_slice()),
            revision: Box::from(b"HEAD".as_slice()),
            targets: Box::new([]),
        }),
    );
    for _ in 0..DECODE_STEPS {
        match output.pop() {
            Some(Output::Request(Request::List { request, .. })) => decoded_step(
                &meter,
                bound,
                &mut domain,
                &env,
                &mut output,
                Some(Event::Listed { request, commit: oid(b'z'), entries: repository.entries.clone().into() }),
            ),
            Some(Output::Request(Request::Read { request, blobs, .. })) => {
                let templates = blobs.iter().any(|blob| blob.kind == As::Template);
                for blob in &blobs {
                    let event = if templates {
                        Event::Read {
                            request,
                            oid: blob.oid.clone(),
                            read: render::vocabulary::Decoded::Template(template.clone()),
                        }
                    } else {
                        parent::blob_event(repository, blob, request, &SHIPPED)
                    };
                    decoded_step(&meter, bound, &mut domain, &env, &mut output, Some(event));
                }
                if templates {
                    return;
                }
            }
            Some(Output::Request(_) | Output::Answer(_)) => panic!("decoded fixture reads only"),
            None => decoded_step(&meter, bound, &mut domain, &env, &mut output, None),
        }
    }
    panic!("the decoded phase reaches its template batch")
}
fn decoded_step(
    meter: &Meter,
    bound: u64,
    domain: &mut render::Render,
    env: &Env<limits::Limits>,
    output: &mut Queue<Output>,
    event: Option<Event>,
) {
    meter.start();
    match event {
        Some(event) => render::step(domain, env, event, output),
        None => render::resume(domain, env, output),
    }
    let step = meter.end();
    meter.check(step, bound, &"decoded arrival");
}

// Ruling 194: each motd/x map is 16,776,933 = 524,277 * 32 + 2 * 32 + 5.
// The other decoded files take 1,065 bytes. Four fewer zeros in each map
// remove 512 bytes, the smallest cut that lets all four fit.
pub const REVISION_ZEROES: usize = 524_277;
const LIST_TYPE: u32 = 1; // Int is the first type, its list the second.
#[must_use]
pub fn large_decoded_revision(extra: bool) -> Repository {
    use render::{
        revision::declaration::{Form, Type},
        vocabulary::Decoded,
    };
    let mut repository = super::with_int_setting();
    let declaration = &mut repository.documents.iter_mut().find(|(id, _)| *id == oid(b'4')).expect("task").1;
    let Decoded::Declaration(declaration) = declaration else { panic!("task declaration") };
    super::only_mut(&mut declaration.settings).ty = LIST_TYPE;
    declaration.types =
        Box::new([Type { at: declaration.at, form: Form::Int }, Type { at: declaration.at, form: Form::List(0) }]);
    for byte in b'a'..=b'd' {
        let path = format!("envs/{}.json", char::from(byte));
        repository.entries.push(entry(path.as_bytes(), byte));
        repository.documents.push((oid(byte), large_decoded_patch()));
    }
    if extra {
        repository.entries.push(entry(b"envs/e.json", b'e'));
        repository.documents.push((oid(b'e'), large_decoded_patch()));
    }
    repository.entries.sort_by(|first, second| first.path.cmp(&second.path));
    repository
}
fn large_decoded_patch() -> render::vocabulary::Decoded {
    use render::revision::{Kind, Value};
    let at = super::Position { line: 1, column: 1 };
    super::task_patch(
        b"motd",
        b"x",
        Kind::List((0..REVISION_ZEROES).map(|_| Value { at, kind: Kind::Int(0) }).collect()),
    )
}

// The existing budget story lists 24-byte files and admits just 64 decoded bytes.
pub const LISTED_CONFIGURATION_BYTES: u64 = 24;
pub const SMALL_DECODED: limits::Limits = limits::Limits { decoded: 64, ..SHIPPED };
pub const DECODE_REQUESTS: usize = 2; // List, then Read before the decoded refusal.
pub fn refuse_document(repository: &mut Repository, byte: u8) {
    super::set_document(
        repository,
        byte,
        render::vocabulary::Decoded::Refused {
            at: super::Position { line: 1, column: 2 },
            fault: render::errors::Decode::Unexpected,
            also: None,
        },
    );
}
