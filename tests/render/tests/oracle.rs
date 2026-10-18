#[path = "oracle/engine.rs"]
mod engine;
use engine::{Oracle, Outcome};
use hearth_render_world::{
    expressions::{self, Style},
    fixtures::{self, oracle as inputs},
    spacing, templates,
};
use inputs::Row;
use std::borrow::Cow;

#[test]
fn generated_expressions_render_as_the_oracle_renders_them() {
    let oracle = Oracle::new();
    for seed in inputs::CASES {
        let (mut rng, roots) = fixtures::draw(inputs::SEED + seed);
        let tree = expressions::tree(&mut rng, &roots, expressions::ORACLE);
        let template = templates::output_of(&expressions::text(&tree, Style::Oracle));
        oracle.compare(&template, &roots).unwrap_or_else(|mismatch| panic!("seed {seed}: {mismatch}"));
    }
}
#[test]
fn each_listed_difference_still_differs_as_listed() {
    Oracle::new().check_differences().expect("listed differences keep their pinned outcomes");
}
const FILTERS: &[Row<Outcome>] = &[
    Row {
        name: "default",
        template: "{{ settings.absent | default('fallback') }}",
        expected: Outcome::Text(Cow::Borrowed("fallback")),
    },
    Row { name: "join", template: "{{ ['a',2] | join(':') }}", expected: Outcome::Text(Cow::Borrowed("a:2")) },
    Row {
        name: "split",
        template: "{{ 'a::b' | split(':') | join('|') }}",
        expected: Outcome::Text(Cow::Borrowed("a||b")),
    },
    Row { name: "first", template: "{{ ['a','b'] | first }}", expected: Outcome::Text(Cow::Borrowed("a")) },
    Row { name: "last", template: "{{ ['a','b'] | last }}", expected: Outcome::Text(Cow::Borrowed("b")) },
    Row { name: "length", template: "{{ 'é水' | length }}", expected: Outcome::Text(Cow::Borrowed("2")) },
    Row { name: "replace", template: "{{ 'aba' | replace('a','x') }}", expected: Outcome::Text(Cow::Borrowed("xbx")) },
    Row {
        name: "items",
        template: "{{ settings | items | tojson }}",
        expected: Outcome::Text(Cow::Borrowed("[[\"a\",2],[\"b\",2]]")),
    },
    Row { name: "keys", template: "{{ settings | keys | join(',') }}", expected: Outcome::Text(Cow::Borrowed("a,b")) },
    Row {
        name: "values",
        template: "{{ settings | values | join(',') }}",
        expected: Outcome::Text(Cow::Borrowed("2,2")),
    },
    Row {
        name: "sort",
        template: "{{ [2,-1,2] | sort | join(',') }}",
        expected: Outcome::Text(Cow::Borrowed("-1,2,2")),
    },
    Row {
        name: "tojson",
        template: "{{ [true,2,'x'] | tojson }}",
        expected: Outcome::Text(Cow::Borrowed("[true,2,\"x\"]")),
    },
    Row {
        name: "shell_quote",
        template: "{{ \"a'b\" | shell_quote }}",
        expected: Outcome::Text(Cow::Borrowed("'a'\\''b'")),
    },
    Row {
        name: "systemd_escape",
        template: "{{ '.a/b-c' | systemd_escape }}",
        expected: Outcome::Text(Cow::Borrowed("\\x2ea-b\\x2dc")),
    },
    Row {
        name: "systemd_escape_path",
        template: "{{ '/a/b-c' | systemd_escape_path }}",
        expected: Outcome::Text(Cow::Borrowed("a-b\\x2dc")),
    },
    Row { name: "basename", template: "{{ '/a/b' | basename }}", expected: Outcome::Text(Cow::Borrowed("b")) },
    Row { name: "dirname", template: "{{ '/a/b' | dirname }}", expected: Outcome::Text(Cow::Borrowed("/a")) },
    Row { name: "root basename", template: "{{ '/' | basename }}", expected: Outcome::Text(Cow::Borrowed("/")) },
    Row { name: "root dirname", template: "{{ '/' | dirname }}", expected: Outcome::Text(Cow::Borrowed("/")) },
    Row {
        name: "absolute leaf dirname",
        template: "{{ '/a' | dirname }}",
        expected: Outcome::Text(Cow::Borrowed("/")),
    },
    Row { name: "relative leaf dirname", template: "{{ 'a' | dirname }}", expected: Outcome::Text(Cow::Borrowed(".")) },
    Row { name: "relative basename", template: "{{ 'a/b' | basename }}", expected: Outcome::Text(Cow::Borrowed("b")) },
    Row { name: "relative dirname", template: "{{ 'a/b' | dirname }}", expected: Outcome::Text(Cow::Borrowed("a")) },
    Row { name: "empty basename", template: "{{ '' | basename }}", expected: Outcome::Fails },
    Row { name: "trailing slash dirname", template: "{{ 'a/' | dirname }}", expected: Outcome::Fails },
    Row { name: "parent basename", template: "{{ '/a/../b' | basename }}", expected: Outcome::Fails },
    Row { name: "relative escape path", template: "{{ 'a/b' | systemd_escape_path }}", expected: Outcome::Fails },
    Row { name: "integer basename", template: "{{ 1 | basename }}", expected: Outcome::Fails },
    Row { name: "integer shell quote", template: "{{ 1 | shell_quote }}", expected: Outcome::Fails },
];
#[test]
fn every_filter_of_7_5_agrees_on_a_pinned_input() {
    inputs::pinned(FILTERS, &Oracle::new(), Oracle::outcomes);
}
const SETTINGS: &[Row<Outcome>] = &[
    Row {
        name: "trim statement line",
        template: "before\n  {% if true %}\nafter{% endif %}",
        expected: Outcome::Text(Cow::Borrowed("before\nafter")),
    },
    Row { name: "keep final newline", template: "last\n", expected: Outcome::Text(Cow::Borrowed("last\n")) },
    Row { name: "strict missing key", template: "{{ settings.missing }}", expected: Outcome::Fails },
    Row { name: "auto escape off", template: "{{ '<>&' }}", expected: Outcome::Text(Cow::Borrowed("<>&")) },
];
#[test]
fn the_oracles_settings_are_the_ones_9_5_names() {
    inputs::pinned(SETTINGS, &Oracle::new(), Oracle::outcomes);
}
#[test]
fn a_difference_not_listed_fails_naming_both_outputs() {
    let mismatch = Oracle::new().compare("{{ true + true }}", &inputs::roots()).expect_err("unlisted coercion differs");
    assert!(mismatch.to_string().contains("hearth Fails, oracle Text(\"2\")"));
    assert_eq!(
        (mismatch.template.as_str(), mismatch.hearth, mismatch.oracle),
        ("{{ true + true }}", Outcome::Fails, Outcome::Text(Cow::Borrowed("2")))
    );
}

#[test]
fn generated_templates_render_as_the_oracle_renders_them() {
    inputs::compare(inputs::CASES, inputs::TEMPLATE_SEED, &Oracle::new(), Oracle::compare, |rng, roots| {
        let tree = templates::tree(rng, roots, templates::ORACLE);
        spacing::dressed(rng, &tree)
    });
}
#[test]
fn markers_on_every_delimiter_render_as_the_oracle_renders_them() {
    let tree = inputs::marker_tree();
    inputs::compare(inputs::CASES, inputs::TEMPLATE_SEED, &Oracle::new(), Oracle::compare, |rng, _| {
        spacing::dressed(rng, &tree)
    });
}
#[test]
fn raw_bodies_render_as_the_oracle_renders_them() {
    inputs::compare(inputs::CASES, inputs::TEMPLATE_SEED, &Oracle::new(), Oracle::compare, |rng, _| spacing::raw(rng));
}
const SCOPES: &[Row<Outcome>] = &[
    Row {
        name: "nested loops restore their parent",
        template: "{% for x in [1,2] %}{{ loop.index }}{% for y in [3,4] %}{{ loop.index0 }}{{ loop.first }}{{ loop.last }}{{ loop.length }}{% set x = y %}{% endfor %}{{ x }}{{ loop.index }}{% endfor %}",
        expected: Outcome::Text(Cow::Borrowed("10truefalse21falsetrue21120truefalse21falsetrue222")),
    },
    Row {
        name: "assignment survives the loop",
        template: "{% set x = 7 %}{% for y in [1,2] %}{% set x = y %}{{ x }}{% endfor %}{{ x }}",
        expected: Outcome::Text(Cow::Borrowed("127")),
    },
    Row {
        name: "branch assignment survives the branch",
        template: "{% set x = 1 %}{% if true %}{% set x = 2 %}{% endif %}{{ x }}",
        expected: Outcome::Text(Cow::Borrowed("2")),
    },
];
#[test]
fn loops_and_assignments_scope_as_the_oracle_scopes_them() {
    inputs::pinned(SCOPES, &Oracle::new(), Oracle::outcomes);
}
#[test]
fn comments_and_trim_blocks_render_as_the_oracle_renders_them() {
    inputs::compare(inputs::CASES, inputs::TEMPLATE_SEED, &Oracle::new(), Oracle::compare, |rng, _| {
        format!("before\n{}\nafter\n", spacing::comment(rng))
    });
}
