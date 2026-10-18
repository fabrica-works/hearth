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
    broken_base: Option<String>,
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
    Unread(String),
    Own(String),
    Exported(String),
    Task,
    Env(String),
    Collide,
}
#[must_use]
pub fn generate(rng: &mut Rng, shape: Shape) -> Generated {
    let mut model = Model {
        targets: Vec::new(),
        broken_base: None,
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
    for base in 0..shape.bases {
        model.documents.push((format!("hosts/b{base}/b{base}.json"), Json::Map(Vec::new())));
    }
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
    let mut model = generated.model.clone();
    match edit {
        Edit::Note | Edit::Reorder | Edit::Respace | Edit::Task => {}
        Edit::Unread(name) | Edit::Own(name) | Edit::Exported(name) => {
            let target = model.targets.iter().find(|target| target.name == *name).expect("model target");
            let path = format!("hosts/{}/{name}.json", target.base);
            let document = &mut model.documents.iter_mut().find(|(p, _)| *p == path).expect("host").1;
            let node = member(document, "node");
            match edit {
                Edit::Unread(_) => increment(member(node, "unread")),
                Edit::Own(_) => increment(member(node, "mtu")),
                Edit::Exported(_) => {
                    let Json::String(name) = member(node, "name") else { panic!("name string") };
                    name.push_str("-edited");
                }
                Edit::Note | Edit::Reorder | Edit::Respace | Edit::Task | Edit::Env(_) | Edit::Collide => {
                    unreachable!("setting edit")
                }
            }
        }
        Edit::Env(environment) => {
            let path = format!("envs/{environment}.json");
            model.documents.iter_mut().find(|(p, _)| *p == path).expect("environment").1 = json::object(&[(
                "debian",
                json::object(&[("pockets", json::object(&[("updates", Json::Bool(true))]))]),
            )]);
        }
        Edit::Collide => {
            for task in ["dup_a", "dup_b"] {
                model.documents.push((format!("tasks/{task}/task.json"), Json::Map(Vec::new())));
                for target in &model.targets {
                    let path = format!("hosts/{}/{}.json", target.base, target.name);
                    let Json::Map(entries) = &mut model.documents.iter_mut().find(|(p, _)| *p == path).expect("host").1
                    else {
                        panic!("host object")
                    };
                    entries.push((task.into(), Json::Map(Vec::new())));
                }
            }
        }
    }
    let mut edited = materialize(rng, model);
    match edit {
        Edit::Task => {
            edited
                .leaves
                .iter_mut()
                .find(|leaf| leaf.path == b"tasks/node/tree/etc/node/mtu.j2")
                .expect("template")
                .bytes
                .extend_from_slice(b"edited\n");
        }
        Edit::Collide => {
            for task in ["dup_a", "dup_b"] {
                edited.leaves.push(Leaf {
                    path: format!("tasks/{task}/tree/etc/dup").into_bytes(),
                    bytes: task.as_bytes().to_vec(),
                    kind: Kind::File,
                });
            }
        }
        Edit::Note
        | Edit::Reorder
        | Edit::Respace
        | Edit::Unread(_)
        | Edit::Own(_)
        | Edit::Exported(_)
        | Edit::Env(_) => {}
    }
    edited
}
fn member<'a>(value: &'a mut Json, name: &str) -> &'a mut Json {
    let Json::Map(entries) = value else { panic!("object") };
    &mut entries.iter_mut().find(|(key, _)| key == name).expect("member").1
}
fn increment(value: &mut Json) {
    let Json::Int(value) = value else { panic!("integer") };
    *value += 1;
}
#[must_use]
pub fn touched(generated: &Generated, edit: &Edit) -> Vec<String> {
    let mut names: Vec<_> = generated
        .model
        .targets
        .iter()
        .filter(|target| match edit {
            Edit::Note | Edit::Reorder | Edit::Respace | Edit::Unread(_) => false,
            Edit::Own(name) => target.name == *name,
            Edit::Exported(name) => {
                target.environment
                    == generated.model.targets.iter().find(|t| t.name == *name).expect("exporter").environment
            }
            Edit::Task | Edit::Collide => true,
            Edit::Env(environment) => target.environment == *environment,
        })
        .map(|target| target.name.clone())
        .collect();
    names.sort();
    names
}
/// Corrupt one base's shared file; all other bases remain valid by construction.
#[must_use]
pub fn mutate(rng: &mut Rng, generated: &Generated) -> Generated {
    let mut mutated = generated.clone();
    let index =
        usize::try_from(rng.below(u64::try_from(generated.model.targets.len()).expect("targets"))).expect("target");
    let base = &generated.model.targets[index].base;
    let path = format!("hosts/{base}/{base}.json");
    let leaf = mutated.leaves.iter_mut().find(|leaf| leaf.path == path.as_bytes()).expect("base document");
    leaf.bytes = hearth_config_world::generate::mutate(rng, &leaf.bytes);
    // A mutator can retain the original; a trailing non-JSON byte guarantees a broken file.
    leaf.bytes.push(b'!');
    mutated.model.broken_base = Some(base.clone());
    mutated
}

/// Targets that compose none of the model's broken files (render plan, ruling 197).
#[must_use]
pub fn healthy(generated: &Generated) -> Vec<String> {
    generated
        .model
        .targets
        .iter()
        .filter(|target| Some(&target.base) != generated.model.broken_base.as_ref())
        .map(|target| target.name.clone())
        .collect()
}

/// The mutation baseline has no environment peer dependency (ruling 197).
#[must_use]
pub fn without_peers(rng: &mut Rng, generated: &Generated) -> Generated {
    let mut model = generated.model.clone();
    let declaration =
        &mut model.documents.iter_mut().find(|(path, _)| path == "tasks/node/task.json").expect("node declaration").1;
    let Json::Map(entries) = declaration else { panic!("declaration object") };
    // A declared export read itself creates the dependency, even without its template.
    entries.retain(|(name, _)| name != "reads");
    let mut isolated = materialize(rng, model);
    isolated.leaves.retain(|leaf| leaf.path != b"tasks/node/tree/etc/node/peers.j2");
    isolated
}
