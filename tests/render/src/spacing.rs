//! Format 7.2's whitespace choices on complete template tokens.
use crate::{
    expressions::Style,
    templates::{self, Text, Tree},
};
use skein_lib::Rng;
const SPACES: &[&str] = &["", " ", "\t", "\n", "\u{000b}", "\u{000c}", "\u{00a0}", "\u{3000}"];
const INSIDE: &[&str] = &[" ", "\t", "\n"];
const OUTPUT: &[&str] = &["", "-"];
const STATEMENT: &[&str] = &["", "-", "+"];
fn choose<'a>(rng: &mut Rng, choices: &'a [&str]) -> &'a str {
    choices[usize::try_from(rng.below(choices.len() as u64)).expect("choice")]
}
fn tag(rng: &mut Rng, open: &str, body: &str, close: &str, markers: &[&str], inside: &[&str]) -> String {
    format!(
        "{}{}{}{}{}{}{}{}{}",
        choose(rng, SPACES),
        open,
        choose(rng, markers),
        choose(rng, inside),
        body,
        choose(rng, inside),
        choose(rng, markers),
        close,
        choose(rng, SPACES)
    )
}
/// Dress a token without changing its expression, statement or literal text.
#[must_use]
pub fn dress(rng: &mut Rng, text: &Text) -> String {
    match text {
        Text::Literal(bytes) => String::from_utf8(bytes.clone()).expect("generated UTF-8"),
        Text::Output(body) => tag(rng, "{{", body, "}}", OUTPUT, INSIDE),
        Text::Statement(body) => tag(rng, "{%", body, "%}", STATEMENT, INSIDE),
    }
}
/// A raw body includes false starts without containing a complete closing tag.
#[must_use]
pub fn raw(rng: &mut Rng) -> String {
    format!(
        "{}{{{{ {{% {{%endraw {}",
        tag(rng, "{%", "raw", "%}", STATEMENT, INSIDE),
        tag(rng, "{%", "endraw", "%}", STATEMENT, INSIDE)
    )
}
#[must_use]
pub fn comment(rng: &mut Rng) -> String {
    tag(rng, "{#", "{{ ' \" {%", "#}", STATEMENT, SPACES)
}
/// Complete trees share the same printer as the reference world's templates.
#[must_use]
pub fn dressed(rng: &mut Rng, tree: &Tree) -> String {
    let mut text = comment(rng);
    text.push_str(&raw(rng));
    for token in templates::tokens(tree, Style::Oracle) {
        text.push_str(&dress(rng, &token));
    }
    text
}

/// Marker and literal constructs actually present in a dressed source.
#[must_use]
pub fn constructs(text: &str) -> std::collections::BTreeSet<&'static str> {
    CONSTRUCTS.iter().copied().filter(|construct| text.contains(construct)).collect()
}
pub const CONSTRUCTS: &[&str] = &[
    "{{-", "-}}", "{%-", "-%}", "{%+", "+%}", "{#-", "-#}", "{#+", "+#}", "raw", "{{ '", "\t", "\n", "\u{000b}",
    "\u{000c}", "\u{00a0}", "\u{3000}",
];
