//! Generated task declarations decoded and expanded through both protocol layers.
use crate::expressions::{self, Roots, Tree as Expression};
use hearth_domain_render::{
    entries,
    errors::Kind,
    limits::Limits,
    revision::declaration::{Map, Name},
};
use skein_lib::{
    Env, Queue, Rng, Time, Wall,
    stream::{Down, Held},
};
#[derive(Clone, Debug)]
pub enum Piece {
    Text(Vec<u8>),
    Output(Expression),
}
#[derive(Clone, Debug)]
pub struct Text(pub Vec<Piece>);
#[derive(Clone, Debug)]
pub enum Literal {
    Text(Text),
    Typed(Expression),
    Int(i64),
    Bool(bool),
    Strings(Vec<Text>),
    Range(Box<Literal>, Box<Literal>),
}
#[derive(Clone, Debug)]
pub struct Declaration {
    pub map: Map,
    pub key: Text,
    pub each: Option<Expression>,
    pub when: Option<Expression>,
    pub fields: Vec<(Name, Literal)>,
}
#[derive(Clone, Debug)]
pub struct Tree(pub Vec<Declaration>);
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Concrete {
    pub key: Vec<u8>,
    pub fields: Vec<(Name, entries::Fielded)>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Group {
    pub map: Map,
    pub entries: Vec<Concrete>,
}
pub type Expansion = Result<Vec<Group>, &'static str>;
fn expression(rng: &mut Rng, roots: &Roots) -> Expression {
    let tree = expressions::tree(rng, roots, expressions::Shape { depth: 1, width: 2 });
    let tree = if let Expression::Output(tree) = tree { *tree } else { tree };
    if expressions::text(&tree).len() > hearth_protocol_template::limits::SHIPPED.expression as usize {
        Expression::Int(42)
    } else {
        tree
    }
}
fn text(rng: &mut Rng, roots: &Roots) -> Text {
    if rng.chance(600) {
        Text(vec![Piece::Text(vec![b'x'; usize::try_from(rng.below(100)).expect("small text")])])
    } else {
        Text(vec![Piece::Output(expression(rng, roots))])
    }
}
#[must_use]
pub fn tree(rng: &mut Rng, roots: &Roots) -> Tree {
    let each = rng
        .chance(700)
        .then(|| Expression::List((0..rng.below(6)).map(|_| Expression::String(b"v".to_vec())).collect()));
    let mut fields = vec![
        (Name::From, Literal::Text(text(rng, roots))),
        (Name::Uid, if rng.chance(700) { Literal::Int(42) } else { Literal::Typed(expression(rng, roots)) }),
        (
            Name::Groups,
            if rng.chance(500) {
                Literal::Strings(vec![text(rng, roots), text(rng, roots)])
            } else {
                Literal::Typed(Expression::List(vec![Expression::String(b"wheel".to_vec())]))
            },
        ),
        (
            Name::Subordinate,
            Literal::Range(
                Box::new(Literal::Int(42)),
                Box::new(if rng.chance(700) { Literal::Int(2) } else { Literal::Typed(expression(rng, roots)) }),
            ),
        ),
    ];
    fields.sort_by_key(|(name, _)| name.key());
    let user_key = if each.is_some() && rng.chance(800) {
        Text(vec![Piece::Text(b"user".to_vec()), Piece::Output(Expression::Name(b"key"))])
    } else {
        Text(vec![Piece::Text(b"user".to_vec())])
    };
    Tree(vec![
        Declaration {
            map: Map::Files,
            key: Text(vec![Piece::Text(b"/etc/example".to_vec())]),
            each: None,
            when: None,
            fields: vec![(
                Name::Directory,
                if rng.chance(800) { Literal::Bool(true) } else { Literal::Typed(expression(rng, roots)) },
            )],
        },
        Declaration {
            map: Map::Users,
            key: user_key,
            each,
            when: rng.chance(250).then(|| expression(rng, roots)),
            fields,
        },
    ])
}
fn tag(expression: &Expression) -> String {
    format!("{{{{ {} }}}}", String::from_utf8(expressions::text(expression)).expect("UTF-8 expression"))
}
fn string(text: &Text) -> String {
    text.0
        .iter()
        .map(|part| match part {
            Piece::Text(text) => String::from_utf8(text.clone()).expect("UTF-8 text"),
            Piece::Output(expression) => tag(expression),
        })
        .collect()
}
fn literal(value: &Literal) -> String {
    match value {
        Literal::Text(text) => format!("{:?}", string(text)),
        Literal::Typed(expression) => format!("{:?}", tag(expression)),
        Literal::Int(value) => value.to_string(),
        Literal::Bool(value) => value.to_string(),
        Literal::Strings(each) => {
            format!("[{}]", each.iter().map(|text| format!("{:?}", string(text))).collect::<Vec<_>>().join(","))
        }
        Literal::Range(base, count) => format!("{{\"base\":{},\"count\":{}}}", literal(base), literal(count)),
    }
}
#[must_use]
pub fn source(tree: &Tree) -> Vec<u8> {
    let mut maps = Vec::new();
    for declaration in &tree.0 {
        let mut fields: Vec<_> = declaration
            .fields
            .iter()
            .map(|(name, value)| {
                format!("{:?}:{}", std::str::from_utf8(name.key()).expect("field key"), literal(value))
            })
            .collect();
        if let Some(each) = &declaration.each {
            fields.push(format!("\"for_each\":{:?}", String::from_utf8(expressions::text(each)).expect("expression")));
        }
        if let Some(when) = &declaration.when {
            fields.push(format!("\"if\":{:?}", String::from_utf8(expressions::text(when)).expect("expression")));
        }
        let map = match declaration.map {
            Map::Files => "files",
            Map::Users => "users",
            Map::Packages
            | Map::Preseeds
            | Map::Initramfs
            | Map::Software
            | Map::Groups
            | Map::Presets
            | Map::Tmpfiles
            | Map::Data
            | Map::Secrets
            | Map::Identity
            | Map::Cmdline
            | Map::Live => unreachable!("generated maps"),
        };
        maps.push(format!("{map:?}:{{{:?}:{{{}}}}}", string(&declaration.key), fields.join(",")));
    }
    format!("{{{}}}", maps.join(",")).into_bytes()
}
#[must_use]
pub fn decode(source: &[u8]) -> hearth_domain_render::revision::declaration::Declaration {
    use hearth_protocol_config::{self as config, json};
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: config::limits::SHIPPED };
    let mut decoder = json::Decoder::new(&env.limits);
    let (mut above, mut below) = (Queue::with_capacity(1), Queue::with_capacity(1));
    let mut held = Held::new(Box::from(source));
    json::down(&mut decoder, &env, json::Request::Decode, &mut above, &mut below);
    for _ in 0..source.len().saturating_mul(8).saturating_add(32) {
        if let Some(event) = above.pop() {
            let json::Event::Decoded(value) = event else { panic!("valid generated JSON: {event:?}") };
            let declaration = config::declaration::declaration(value, &env.limits).expect("generated declaration");
            return hearth_protocol_builder::translate::declarations::declaration(
                declaration,
                &hearth_protocol_builder::limits::SHIPPED,
            )
            .unwrap_or_else(|error| panic!("generated expressions: {error:?}: {}", String::from_utf8_lossy(source)));
        }
        if let Some(Down::Demand { read, room: 0 }) = below.pop() {
            json::up(&mut decoder, &env, held.answer(read).expect("held source"), &mut above, &mut below);
        }
    }
    panic!("decoder did not settle")
}
/// # Errors
/// Returns the first expansion fault after decoding the generated declaration.
pub fn expand(tree: &Tree, roots: &Roots, limits: &Limits) -> Expansion {
    let declaration = decode(&source(tree));
    entries::expand(&declaration, &roots.context(), limits, &mut { limits.expanded })
        .map(|expanded| {
            expanded
                .groups
                .into_iter()
                .map(|group| Group {
                    map: group.map,
                    entries: group
                        .entries
                        .into_iter()
                        .map(|entry| Concrete {
                            key: entry.key.into_vec(),
                            fields: entry.fields.into_iter().map(|field| (field.name, field.value)).collect(),
                        })
                        .collect(),
                })
                .collect()
        })
        .map_err(|fault| match fault.kind {
            Kind::Evaluation(kind) => crate::fixtures::kind(kind),
            Kind::Schema(_) => "wrong",
            Kind::Collision(_) => "collision",
            Kind::Ids(_)
            | Kind::Store(_)
            | Kind::Decode(_)
            | Kind::Repository(_)
            | Kind::Resolution(_)
            | Kind::Tool(_)
            | Kind::Composition(_)
            | Kind::Guard(_)
            | Kind::Exported(_) => unreachable!("expansion faults"),
        })
}
