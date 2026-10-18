//! Shared roots and allocator checks for the memory stories.
use crate::expressions::Roots;
use skein_world::domain::heap::Meter;
/// Empty lexical bindings contain no caller-owned values.
pub const EMPTY_BINDINGS: u32 = 0;
/// A MiB output still reaches the configured output peak; all evaluator,
/// binding and made-byte limits retain their shipped values.
pub const TEMPLATE_LIMITS: hearth_domain_render::limits::Limits = hearth_domain_render::limits::Limits {
    rendered_file: hearth_domain_render::limits::SHIPPED.rendered_file / 16,
    ..hearth_domain_render::limits::SHIPPED
};

pub fn each<T>(bound: u64, rows: impl IntoIterator<Item = (T, &'static str)>, run: impl Fn(&T, &Roots)) {
    let roots = super::wide_roots();
    for (input, what) in rows {
        peak(&Meter::new(), bound, &what, || run(&input, &roots));
    }
}
pub fn peak(meter: &Meter, bound: u64, what: &dyn core::fmt::Debug, run: impl FnOnce()) {
    meter.start();
    run();
    let measured = meter.end();
    meter.check(measured, bound, what);
}

/// The evaluator keeps its configured peaks. Repeating assignment to x
/// retains one binding; a single assignment reaches that same allocation.
const NESTED_LOOPS: usize = hearth_domain_render::limits::SHIPPED.statements as usize;
// Twenty one-MiB operands exceed the shipped sixteen-MiB made budget.
const CONCAT_OPERANDS: usize = 20;
#[must_use]
pub fn templates_at_the_limits() -> [(hearth_domain_render::evaluate::template::Template, &'static str); 4] {
    let loops = format!(
        "{}{{{{ loop.index }}}}{}",
        "{% for x in [1] %}".repeat(NESTED_LOOPS),
        "{% endfor %}".repeat(NESTED_LOOPS)
    );
    let output = "{% for x in [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16] %}{{ settings.text }}{% endfor %}";
    let chain = format!("{{{{ {} }}}}", vec!["settings.text"; CONCAT_OPERANDS].join(" ~ "));
    let sets = compiled_template("{% set x = 1 %}");
    [
        (compiled_template(&loops), "32 nested loops"),
        (compiled_template(output), "output budget"),
        (sets, "set bindings"),
        (compiled_template(&chain), "concat past made"),
    ]
}
fn compiled_template(source: &str) -> hearth_domain_render::evaluate::template::Template {
    let compiled =
        hearth_protocol_template::template::compile(source.as_bytes(), &hearth_protocol_template::limits::SHIPPED)
            .expect("limit template compiles");
    hearth_protocol_builder::translate::instructions::template(compiled)
}

// Sixty-three one-MiB fields fit below the expanded 64-MiB allowance,
// leaving space for their entry and field records and keys.
const EXPANDED_COPIES: u64 =
    hearth_domain_render::limits::SHIPPED.expanded / hearth_domain_render::limits::SHIPPED.document as u64 - 1;
#[must_use]
pub fn expanded_template()
-> (hearth_domain_render::revision::declaration::Declaration, hearth_domain_render::evaluate::template::Template) {
    let entries = (0..EXPANDED_COPIES)
        .map(|index| format!(r#""g{index}":{{"from":"{{{{ settings.text }}}}"}}"#))
        .collect::<Vec<_>>()
        .join(",");
    let declaration = crate::declarations::decode(format!(r#"{{"groups":{{{entries}}}}}"#).as_bytes());
    (declaration, compiled_template("{{ settings.text }}"))
}
