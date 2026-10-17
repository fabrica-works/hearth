use hearth_domain_render::{
    parts::config::Node,
    revision,
    vocabulary::{Answer, Decoded},
};
use hearth_lib::encode::Digest;
use hearth_render_world::fixtures;
use std::fmt::Write;
fn integer(value: &revision::Value) -> Option<i64> {
    match &value.kind {
        revision::Kind::Int(value) => Some(*value),
        revision::Kind::Map(pairs) => pairs.iter().find_map(|pair| integer(&pair.value)),
        revision::Kind::List(items) => items.iter().find_map(integer),
        revision::Kind::Bool(_) | revision::Kind::String(_) | revision::Kind::Null => None,
    }
}
fn peers(repository: &fixtures::Repository, environment: &[u8]) -> Vec<u8> {
    let mut peers = String::new();
    for entry in &repository.entries {
        let revision::Class::Host { base: _, name } = revision::class(&entry.path) else { continue };
        if name.rsplit(|byte| *byte == b'-').next() != Some(environment) {
            continue;
        }
        let (_, Decoded::Document(document)) =
            repository.documents.iter().find(|(oid, _)| *oid == entry.oid).expect("host")
        else {
            panic!("host document")
        };
        if let Some(value) =
            document.layer.tasks.iter().find(|task| task.name.as_ref() == b"vpn").and_then(|task| integer(&task.value))
        {
            write!(peers, "{}={value};", String::from_utf8_lossy(name)).expect("reference output");
        }
    }
    peers.push('\n');
    peers.into_bytes()
}
#[test]
fn generated_fleets_give_each_reader_its_environments_exporters() {
    for seed in 0..200_u64 {
        let repository = fixtures::exports::fleet(0x0020_0000 + seed);
        let run = fixtures::render_targets(&repository, &[b"a-lab", b"a-prod"], seed);
        assert_eq!(run.answers.len(), 2, "seed {seed}: each reader renders");
        for answer in &run.answers {
            let Answer::Spec(spec) = answer else { panic!("seed {seed}: {answer:?}") };
            let entry = fixtures::only(&fixtures::config_part(&spec.parts).tree);
            let Node::File { content_digest, .. } = entry.node else { panic!("file") };
            assert_eq!(
                content_digest,
                Digest(skein_lib::sha256::digest(&peers(&repository, &spec.meta.environment))),
                "seed {seed}: {:?}",
                spec.meta.target
            );
        }
    }
}
