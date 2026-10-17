use super::{Repository, document, entry, oid, placement, task_patch};
use hearth_domain_render::{
    revision::{self, Position},
    vocabulary::Decoded,
};
use skein_lib::Rng;
const DECLARATIONS: &[(&[u8], &[u8])] = &[
    (b"tasks/vpn/task.json", br#"{"settings":{"x":{"type":"int","doc":"X."}},"exports":{"port":{"type":"int","value":"{{ settings.x }}","doc":"Port."}}}"#),
    (b"tasks/reader/task.json", br#"{"reads":{"vpn":{"exports":true}}}"#),
];
#[must_use]
pub fn fleet(seed: u64) -> Repository {
    let mut rng = Rng::new(seed);
    let mut repository = Repository { entries: Vec::new(), blobs: Vec::new(), documents: Vec::new() };
    for (path, text) in DECLARATIONS {
        let byte = u8::try_from(repository.entries.len()).expect("small fleet");
        repository.entries.push(entry(path, byte));
        repository.documents.push((oid(byte), Decoded::Declaration(crate::declarations::decode(text))));
    }
    for environment in ["lab", "prod"] {
        let byte = u8::try_from(repository.entries.len()).expect("small fleet");
        repository.entries.push(entry(format!("envs/{environment}.json").as_bytes(), byte));
        repository.documents.push((oid(byte), document(None)));
        for base in ["a", "b"] {
            let byte = u8::try_from(repository.entries.len()).expect("small fleet");
            let Decoded::Document(mut host) = document(Some(placement())) else { panic!("document") };
            host.layer.debian = None;
            let at = Position { line: 1, column: 1 };
            let mut tasks = vec![revision::Task {
                name: b"reader".as_slice().into(),
                at,
                value: revision::Value { at, kind: revision::Kind::Map(Box::new([])) },
            }];
            if rng.below(3) != 0 {
                let Decoded::Document(exporter) =
                    task_patch(b"vpn", b"x", revision::Kind::Int(i64::try_from(rng.below(100)).expect("port")))
                else {
                    panic!("document")
                };
                tasks.extend(exporter.layer.tasks);
            }
            host.layer.tasks = tasks.into_boxed_slice();
            repository.entries.push(entry(format!("hosts/{base}/{base}-{environment}.json").as_bytes(), byte));
            repository.documents.push((oid(byte), Decoded::Document(host)));
        }
    }
    let byte = u8::try_from(repository.entries.len()).expect("small fleet");
    let text: &[u8] = b"{% for name, peer in exports.vpn | items %}{{ name }}={{ peer.port }};{% endfor +%}\n";
    let mut template = entry(b"tasks/reader/tree/etc/peers.j2", byte);
    template.size = u64::try_from(text.len()).expect("template");
    repository.entries.push(template);
    repository.blobs.push((oid(byte), text.into()));
    repository.entries.sort_by(|a, b| a.path.cmp(&b.path));
    repository
}
