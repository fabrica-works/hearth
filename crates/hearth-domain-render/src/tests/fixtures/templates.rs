//! Input builders for flat template programs.
use crate::{
    evaluate::{
        Context, Expression, Fault, Operation,
        template::{self, Does, Instruction, Names, Template},
    },
    limits::{self, Limits},
};
use alloc::boxed::Box;
use skein_lib::bytes;
pub fn template<const N: usize>(lines: &[u32], instructions: [Instruction; N]) -> Template {
    Template { lines: lines.into(), instructions: Box::new(instructions) }
}
pub fn text(at: u32, bytes: &[u8]) -> Instruction {
    Instruction { at, does: Does::Text(bytes::copy_of(bytes)) }
}
pub fn output(at: u32, expression: Expression) -> Instruction {
    Instruction { at, does: Does::Output(expression) }
}
pub fn branch(at: u32, condition: Expression, over: u32) -> Instruction {
    Instruction { at, does: Does::Branch { condition, over } }
}
pub fn jump(at: u32, over: u32) -> Instruction {
    Instruction { at, does: Does::Jump { over } }
}
pub fn enter(at: u32, names: Names, iterable: Expression, over: u32) -> Instruction {
    Instruction { at, does: Does::Enter { names, iterable, over } }
}
pub fn advance(at: u32, back: u32) -> Instruction {
    Instruction { at, does: Does::Advance { back } }
}
pub fn set(at: u32, name: &[u8], value: Expression) -> Instruction {
    Instruction { at, does: Does::Set { name: bytes::copy_of(name), value } }
}
pub fn one(name: &[u8]) -> Names {
    Names::One { name: bytes::copy_of(name) }
}
pub fn pair(first: &[u8], second: &[u8]) -> Names {
    Names::Pair { first: bytes::copy_of(first), second: bytes::copy_of(second) }
}
pub fn expr<const N: usize>(values: u32, operations: [Operation; N]) -> Expression {
    Expression { values, operations: Box::new(operations) }
}
pub fn run(roots: &super::Roots, template: &Template) -> Result<Box<[u8]>, Fault> {
    run_at(roots, template, &limits::SHIPPED)
}
pub fn run_at(roots: &super::Roots, template: &Template, limits: &Limits) -> Result<Box<[u8]>, Fault> {
    let context: Context<'_> = roots.context();
    template::run(template, &context, None, limits, u64::from(limits.rendered_file))
}

pub fn keeping_templates() -> (crate::Render, skein_lib::Env<Limits>, skein_lib::Queue<crate::vocabulary::Output>) {
    let mut side = super::shared_template_call();
    side.decoded(
        &crate::revision::Oid(bytes::copy_of(b"template")),
        crate::vocabulary::Decoded::Template(template(&[0], [text(0, b"abc")])),
    );
    side.rendering = Some(crate::call::templates::Target::new(
        super::meta(),
        Box::new([]),
        super::digest("0101010101010101010101010101010101010101010101010101010101010101"),
        crate::schema::Validated {
            tasks: Box::new([crate::schema::Checked {
                name: bytes::copy_of(b"motd"),
                settings: super::Value::Map(Box::new([])),
                tasks: super::Value::Map(Box::new([])),
            }]),
        },
        super::Roots::x1_prod().debian,
        template_prepared(
            super::kinds::template_plans(&side.entries),
            crate::entries::Expansion { tasks: Box::new([]) },
        ),
        5,
    ));
    side.targets = Box::new([]);
    side.ready = true;
    let render = crate::Render { call: Some(crate::call::Call { side, diff: None }) };
    (
        render,
        skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: limits::SHIPPED },
        skein_lib::Queue::with_capacity(1),
    )
}

pub const TEMPLATE_TARGET_LIMIT: u64 = 3;

pub fn target_limited_answers() -> Box<[crate::vocabulary::Answer]> {
    use crate::vocabulary::{Event, Output, Request};
    let (mut render, mut env, mut out) = keeping_templates();
    env.limits.rendered_target = TEMPLATE_TARGET_LIMIT;
    render.call.as_mut().expect("call").side.limits = env.limits;
    crate::resume(&mut render, &env, &mut out);
    let Some(Output::Request(Request::Keep { request, sha256, .. })) = out.pop() else { panic!("first keep") };
    crate::step(&mut render, &env, Event::Kept { request, sha256 }, &mut out);
    super::finish_render(&mut render, &env, &mut out)
}

pub fn template_prepared(
    plans: Box<[crate::kinds::files::Plan]>,
    expansion: crate::entries::Expansion,
) -> crate::call::templates::Prepared {
    crate::call::templates::Prepared {
        boot: super::built_boot(&[]),
        plans,
        expansion,
        secrets: crate::parts::secrets::Secrets {
            files: Box::new([]),
            passwords: Box::new([]),
            identity: Box::new([]),
        },
        packages: super::thin_packages(),
        live: crate::parts::live::Live { paths: Box::new([]) },
        exports: crate::parts::exports::build(Box::new([])),
        export_roots: crate::evaluate::Value::Map(Box::new([])),
    }
}

/// Drives internal progress until the next answer.
pub fn answer(call: &mut crate::call::Side) -> crate::vocabulary::Answer {
    for _ in 0_u32..100 {
        if let Some(answer) = call.answer() {
            return answer;
        }
        let mut out = skein_lib::Queue::with_capacity(1);
        call.template_request(&mut out);
        if let Some(crate::vocabulary::Output::Request(crate::vocabulary::Request::Keep { sha256, .. })) = out.pop() {
            call.kept(Some(sha256));
        }
    }
    panic!("fixture answer bound")
}
