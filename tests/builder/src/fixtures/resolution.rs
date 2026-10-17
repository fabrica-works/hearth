use super::File;
use hearth_lib::encode::Item;
use hearth_protocol_tree::resolution::{Costs, Inputs, Line, Made, Package, Resolution};
use skein_lib::{Duration, Wall};
pub const TEST_MIN_KEY: &str = "21e9a02feb654883ef051d247cb14db878e04c03c2f7675c784a679ea0956d5d";
pub const TEST_MIN_KEYS_LINE: &str = "keys         packages 46a0bdef5234   software 2dcc0e90ef26   config 408d4c837487";
pub const TEST_MIN_DIGEST: hearth_lib::encode::Digest = hearth_lib::encode::Digest([
    0x90, 0x8a, 0x3b, 0x8a, 0x1d, 0xdd, 0x78, 0x10, 0x0f, 0x9f, 0x44, 0xb2, 0xd3, 0xe6, 0xb3, 0xf4, 0xed, 0xfd, 0x65,
    0xb2, 0x26, 0xdf, 0x34, 0x8e, 0x50, 0x21, 0xb5, 0xd6, 0xd3, 0x7f, 0x63, 0x8b,
]);
pub const TEST_REPOSITORY: &[File] = &[
    File { path: "envs/boot.json", body: include_bytes!("../../../../testing/repos/test/envs/boot.json"), mode: 0o644 },
    File { path: "envs/min.json", body: include_bytes!("../../../../testing/repos/test/envs/min.json"), mode: 0o644 },
    File { path: "fleet.json", body: include_bytes!("../../../../testing/repos/test/fleet.json"), mode: 0o644 },
    File {
        path: "hosts/test/test-min.json",
        body: include_bytes!("../../../../testing/repos/test/hosts/test/test-min.json"),
        mode: 0o644,
    },
    File {
        path: "hosts/test/test.json",
        body: include_bytes!("../../../../testing/repos/test/hosts/test/test.json"),
        mode: 0o644,
    },
    File {
        path: "hosts/test/test-boot.json",
        body: include_bytes!("../../../../testing/repos/test/hosts/test/test-boot.json"),
        mode: 0o644,
    },
    File {
        path: "tasks/hearth/task.json",
        body: include_bytes!("../../../../testing/repos/test/tasks/hearth/task.json"),
        mode: 0o644,
    },
    File {
        path: "tasks/boot/task.json",
        body: include_bytes!("../../../../testing/repos/test/tasks/boot/task.json"),
        mode: 0o644,
    },
];
#[must_use]
pub fn test_min_recorded() -> Resolution {
    Resolution {
        key: hearth_lib::encode::Digest([
            0x21, 0xe9, 0xa0, 0x2f, 0xeb, 0x65, 0x48, 0x83, 0xef, 0x05, 0x1d, 0x24, 0x7c, 0xb1, 0x4d, 0xb8, 0x78, 0xe0,
            0x4c, 0x03, 0xc2, 0xf7, 0x67, 0x5c, 0x78, 0x4a, 0x67, 0x9e, 0xa0, 0x95, 0x6d, 0x5d,
        ]),
        digest: TEST_MIN_DIGEST,
        inputs: Inputs {
            apt: Box::new([Item::Map(1), text(b"install_recommends"), Item::False]),
            request: Box::new([Item::List(0)]),
            variant: Box::new([text(b"apt")]),
            sources: Box::new([
                Item::Map(5),
                text(b"architecture"),
                text(b"amd64"),
                text(b"archive"),
                text(b"20261017T143929Z"),
                text(b"components"),
                Item::List(1),
                text(b"main"),
                text(b"suite"),
                text(b"forky"),
                text(b"updates"),
                Item::False,
            ]),
        },
        closure: Box::new([Package {
            name: b"apt".as_slice().into(),
            architecture: b"amd64".as_slice().into(),
            version: b"3.3.3".as_slice().into(),
            source: 0,
            filename: b"pool/main/a/apt/apt_3.3.3_amd64.deb".as_slice().into(),
            size: 1_567_912,
            sha256: hearth_lib::encode::Digest([
                0xc9, 0xc5, 0x69, 0x7b, 0x1b, 0xfa, 0x54, 0xd5, 0x70, 0x63, 0x72, 0x49, 0xb7, 0x8a, 0xee, 0xd3, 0x75,
                0x51, 0x01, 0x4f, 0xdf, 0xb6, 0x6d, 0xc4, 0x33, 0x48, 0x27, 0xf7, 0xc9, 0xb5, 0x47, 0x16,
            ]),
        }]),
        sources: Box::new([Line {
            uri: b"http://snapshot.debian.org/archive/debian/20261017T143929Z/".as_slice().into(),
            suite: b"forky".as_slice().into(),
            components: Box::new([b"main".as_slice().into()]),
            indices: Box::new([]),
        }]),
        made: Made {
            at: Wall::EPOCH,
            apt: b"".as_slice().into(),
            mmdebstrap: b"".as_slice().into(),
            costs: Costs { fetched: 0, took: Duration::ZERO },
        },
    }
}
fn text(bytes: &[u8]) -> Item {
    Item::Bytes(bytes.into())
}
#[must_use]
pub fn spec_packages_digests(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .filter(|line| line.contains("\"record\":\"spec\"") || line.contains("\"name\":\"packages\""))
        .map(|line| {
            line.split_once("\"digest\":\"").expect("digest field").1.split_once('"').expect("digest text").0.to_owned()
        })
        .collect()
}

#[must_use]
pub fn recorded_test_min(label: &str) -> super::Repository {
    let repository = super::Repository::make(label, TEST_REPOSITORY);
    record_test_min(&repository);
    repository
}
pub fn record_test_min(repository: &super::Repository) {
    super::lay(&repository.scratch().path().join("store"), &[super::Laid::Resolution(test_min_recorded())]);
}

#[must_use]
pub fn dead_proxy() -> Box<[u8]> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("loopback port");
    format!("http://{}", listener.local_addr().expect("bound address")).into_bytes().into()
}

use super::{Call, Form, Release, Rendered, Repository};
use std::os::unix::ffi::OsStrExt;
pub(super) fn call_at(repository: &Repository, at: &str, targets: &[&[u8]], resolve: bool) -> Call {
    Call::Render {
        resolve,
        repository: repository.work_tree.join(".git").as_os_str().as_bytes().into(),
        revision: at.as_bytes().into(),
        targets: targets.iter().map(|target| Box::<[u8]>::from(*target)).collect(),
    }
}
#[must_use]
pub fn resolve(repository: &Repository, targets: &[&[u8]], form: Form, proxy: &[u8]) -> Rendered {
    super::run_holding(
        repository,
        call_at(repository, "HEAD", targets, true),
        form,
        None,
        &hearth::limits(),
        Release::After(skein_lib::Duration::ZERO),
        Some(proxy.into()),
    )
    .rendered
}

#[must_use]
pub fn unknown_package_repository() -> Repository {
    let repository = Repository::make("unknown-package", TEST_REPOSITORY);
    drop(repository.next(
        &[
            File { path: "tasks/absent/task.json", body: br#"{"packages":{"hearth-absent":{}}}"#, mode: 0o644 },
            File { path: "hosts/test/test-min.json", body: br#"{"absent":{}}"#, mode: 0o644 },
        ],
        &[],
    ));
    repository
}
