pub const ABC_SHA256: hearth_lib::encode::Digest = hearth_lib::encode::Digest([
    186, 120, 22, 191, 143, 1, 207, 234, 65, 65, 64, 222, 93, 174, 34, 35, 176, 3, 97, 163, 150, 23, 122, 156, 180, 16,
    255, 97, 242, 0, 21, 173,
]);
use crate::parts::{live, packages, secrets, software};
use skein_lib::bytes::copy_of;
pub fn changed_download() -> (software::Software, software::Software) {
    let before = super::software(&[]);
    let mut after = before.clone();
    super::only_mut(&mut after.downloads).sha256 = hearth_lib::encode::Digest([2; 32]);
    (before, after)
}
pub fn changed_secrets() -> (secrets::Secrets, secrets::Secrets) {
    let before = secrets::Secrets {
        files: Box::new([super::secret_file(b"key", b"/etc/key", 0o600, b"root", b"root")]),
        passwords: Box::new([secrets::Password { user: copy_of(b"free"), secret: copy_of(b"users/free.passwd") }]),
        identity: Box::new([secrets::Identity {
            name: copy_of(b"ssh"),
            kind: secrets::IdentityKind::SshHostKey { algorithm: secrets::Algorithm::Ed25519 },
            path: copy_of(b"/etc/ssh/key"),
            public_path: None,
            mode: 0o600,
            owner: copy_of(b"root"),
            group: copy_of(b"root"),
        }]),
    };
    let mut after = before.clone();
    super::only_mut(&mut after.identity).public_path = Some(copy_of(b"/etc/ssh/key.pub"));
    super::only_mut(&mut after.files).mode = 0o640;
    super::only_mut(&mut after.passwords).secret = copy_of(b"users/free2.passwd");
    (before, after)
}
pub fn changed_live() -> (live::Live, live::Live) {
    let before =
        live::build(&super::live_entry(b"reload", Some(b"postfix.service"), &[b"/etc/mailname"])).expect("live");
    let mut after = before.clone();
    let consumer = live::Consumer { task: copy_of(b"other"), unit: None, action: live::Action::None };
    let path = super::only_mut(&mut after.paths);
    path.consumers = Box::new([super::only(&path.consumers).clone(), consumer]);
    (before, after)
}
pub fn changed_seeds() -> (packages::Packages, packages::Packages) {
    let mut before = super::thin_packages();
    before.seeds = packages::Seeds {
        users: Box::new([
            packages::SeedUser { name: copy_of(b"changed"), uid: 120, gid: None },
            packages::SeedUser { name: copy_of(b"removed"), uid: 120, gid: Some(121) },
            packages::SeedUser { name: copy_of(b"same"), uid: 120, gid: None },
        ]),
        groups: Box::new([
            packages::SeedGroup { name: copy_of(b"changed"), gid: 120 },
            packages::SeedGroup { name: copy_of(b"removed"), gid: 120 },
            packages::SeedGroup { name: copy_of(b"same"), gid: 120 },
        ]),
    };
    let mut after = before.clone();
    after.seeds = packages::Seeds {
        users: Box::new([
            packages::SeedUser { name: copy_of(b"same"), uid: 120, gid: None },
            packages::SeedUser { name: copy_of(b"changed"), uid: 121, gid: Some(121) },
            packages::SeedUser { name: copy_of(b"added"), uid: 122, gid: Some(123) },
        ]),
        groups: Box::new([
            packages::SeedGroup { name: copy_of(b"same"), gid: 120 },
            packages::SeedGroup { name: copy_of(b"changed"), gid: 121 },
            packages::SeedGroup { name: copy_of(b"added"), gid: 122 },
        ]),
    };
    (before, after)
}

pub fn package_pair() -> (packages::Packages, packages::Packages) {
    let before = super::thin_packages();
    let after = before.clone();
    (before, after)
}

pub fn failed_hunk_call()
-> (crate::Render, skein_lib::Env<crate::limits::Limits>, skein_lib::Queue<crate::vocabulary::Output>) {
    let (mut render, env, mut out) = super::hunk_call();
    crate::step(&mut render, &env, super::hunk_failure(), &mut out);
    (render, env, out)
}
pub fn compared_specs(before: crate::parts::Spec, after: crate::parts::Spec) -> crate::diff::TargetChange {
    use crate::diff::{self, Made, Needs};
    diff::compare(Some(&Made::Spec(Box::new(before))), Some(&Made::Spec(Box::new(after))), Needs::Needed)
}
