//! Small fleets generated from a model, never from a rendered spec (render.md, 9.3).
use crate::{
    golden::{Kind, Leaf},
    json::{self, Json},
};
use skein_lib::Rng;
#[derive(Clone, Copy, Debug)]
pub struct Shape {
    pub bases: u32,
}
impl Shape {
    pub const SMALL: Shape = Shape { bases: 3 };
}
#[derive(Clone, Debug)]
pub struct Target {
    pub name: String,
    pub base: String,
    pub environment: String,
}
#[derive(Clone, Debug)]
pub struct Model {
    pub targets: Vec<Target>,
    documents: Vec<(String, Json)>,
}
#[derive(Clone, Debug)]
pub struct Generated {
    pub leaves: Vec<Leaf>,
    pub model: Model,
}
#[derive(Clone, Debug)]
pub enum Edit {
    Note,
    Reorder,
    Respace,
}
#[must_use]
pub fn generate(rng: &mut Rng, shape: Shape) -> Generated {
    let mut model = Model {
        targets: Vec::new(),
        documents: vec![
            (
                "fleet.json".into(),
                json::object(&[(
                    "debian",
                    json::object(&[
                        ("suite", json::string("forky")),
                        ("architecture", json::string("amd64")),
                        ("snapshot", json::object(&[("archive", json::string("20261001T030405Z"))])),
                        ("pockets", json::object(&[("updates", Json::Bool(false)), ("security", Json::Bool(false))])),
                    ]),
                )]),
            ),
            ("tasks/hearth/task.json".into(), Json::Map(Vec::new())),
            ("tasks/node/task.json".into(), declaration()),
        ],
    };
    for environment in ["prod", "lab"] {
        model.documents.push((format!("envs/{environment}.json"), Json::Map(Vec::new())));
        for base in 0..shape.bases {
            let base = format!("b{base}");
            let name = format!("{base}-{environment}");
            model.documents.push((
                format!("hosts/{base}/{name}.json"),
                json::object(&[
                    (
                        "placement",
                        json::object(&[
                            ("kind", json::string("metal")),
                            ("firmware", json::string("efi")),
                            ("hardware", json::string("generated")),
                        ]),
                    ),
                    (
                        "node",
                        json::object(&[
                            ("mtu", Json::Int(1200 + i64::try_from(rng.below(800)).expect("mtu"))),
                            ("name", json::string(&format!("{name}-{}", rng.below(1_000_000)))),
                            ("unread", Json::Int(i64::try_from(rng.below(1_000_000)).expect("setting"))),
                        ]),
                    ),
                ]),
            ));
            model.targets.push(Target { name, base, environment: environment.into() });
        }
    }
    materialize(rng, model)
}
fn declaration() -> Json {
    let setting = |kind| json::object(&[("type", json::string(kind)), ("doc", json::string("Generated setting."))]);
    json::object(&[
        ("settings", json::object(&[("mtu", setting("int")), ("name", setting("string")), ("unread", setting("int"))])),
        ("reads", json::object(&[("node", json::object(&[("exports", Json::Bool(true))]))])),
        (
            "exports",
            json::object(&[(
                "address",
                json::object(&[
                    ("type", json::string("string")),
                    ("value", json::string("{{ settings.name }}")),
                    ("doc", json::string("Generated address.")),
                ]),
            )]),
        ),
    ])
}
fn materialize(rng: &mut Rng, model: Model) -> Generated {
    let mut leaves: Vec<_> = model
        .documents
        .iter()
        .map(|(path, value)| Leaf { path: path.as_bytes().to_vec(), bytes: json::print(rng, value), kind: Kind::File })
        .collect();
    leaves.extend([
        Leaf {
            path: b"tasks/node/tree/etc/node/mtu.j2".to_vec(),
            bytes: b"{{ settings.mtu }}\n".to_vec(),
            kind: Kind::File,
        },
        Leaf {
            path: b"tasks/node/tree/etc/node/peers.j2".to_vec(),
            bytes: b"{{ exports.node | tojson }}\n".to_vec(),
            kind: Kind::File,
        },
    ]);
    Generated { leaves, model }
}
#[must_use]
pub fn edit(rng: &mut Rng, generated: &Generated, edit: &Edit) -> Generated {
    match edit {
        Edit::Note | Edit::Reorder | Edit::Respace => materialize(rng, generated.model.clone()),
    }
}
#[must_use]
pub fn touched(_: &Generated, edit: &Edit) -> Vec<String> {
    match edit {
        Edit::Note | Edit::Reorder | Edit::Respace => Vec::new(),
    }
}
