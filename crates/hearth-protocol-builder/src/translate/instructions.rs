//! Template instructions translated in full (render.md, 8.2).
use crate::render::evaluate::template as render;
use hearth_protocol_template::instruction as template;
use skein_lib::List;
#[must_use]
pub fn template(template: template::Template) -> render::Template {
    let template::Template { instructions, lines } = template;
    let mut translated = List::with_capacity(u32::try_from(instructions.len()).expect("bounded instructions"));
    for item in instructions {
        translated.push(instruction(item)).expect("instruction room");
    }
    render::Template { instructions: translated.into_boxed(), lines }
}
#[must_use]
pub fn instruction(item: template::Instruction) -> render::Instruction {
    let template::Instruction { at, does: value } = item;
    render::Instruction { at, does: does(value) }
}
#[must_use]
pub fn does(value: template::Does) -> render::Does {
    use super::operations::expression;
    match value {
        template::Does::Text(bytes) => render::Does::Text(bytes),
        template::Does::Output(value) => render::Does::Output(expression(value)),
        template::Does::Branch { condition, over } => render::Does::Branch { condition: expression(condition), over },
        template::Does::Jump { over } => render::Does::Jump { over },
        template::Does::Enter { names: bound, iterable, over } => {
            render::Does::Enter { names: names(bound), iterable: expression(iterable), over }
        }
        template::Does::Advance { back } => render::Does::Advance { back },
        template::Does::Set { name, value } => render::Does::Set { name, value: expression(value) },
    }
}
#[must_use]
pub fn names(value: template::Names) -> render::Names {
    match value {
        template::Names::One { name } => render::Names::One { name },
        template::Names::Pair { first, second } => render::Names::Pair { first, second },
    }
}
