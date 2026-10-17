use hearth_conformance_world::resolve::{self, Resolve};
use hearth_protocol_git::invocation::Exit;
use hearth_protocol_mmdebstrap::{
    limits::{self, Limits},
    vector,
};

#[test]
fn real_mmdebstrap_resolves_test_min_through_the_mirror() {
    let (scenario, case) = (Resolve::new(), resolve::case("test-min"));
    let resolved = scenario.run(case);
    assert_eq!(resolved.run.exit, Exit::Code(0), "{}", resolved.run.stderr_text());
    let insts = resolved.inst_lines();
    assert!(insts.len() > 50 && insts.iter().any(|line| line.starts_with(b"Inst apt ")), "the essential set and apt");
    let list = resolved.list("main").expect("apt left main's Packages list");
    assert!(list == scenario.index(), "apt's list is the checked index byte for byte");
    let mut recording = resolved.recording();
    recording.stanzas = Some(resolve::stanzas(&list, &insts).expect("a stanza for each Inst line"));
    scenario.settle(case, &recording);
}
const BOOT_REQUEST: &[&[u8]] = &[
    b"efibootmgr",
    b"git",
    b"incus-agent",
    b"linux-image-amd64",
    b"mmdebstrap",
    b"openssh-server",
    b"systemd",
    b"systemd-container",
    b"tiny-initramfs",
    b"uidmap",
];
#[test]
fn real_mmdebstrap_resolves_each_package_test_boot_requests() {
    let (scenario, case) = (Resolve::new(), resolve::case("test-boot"));
    let resolved = scenario.run(case);
    assert_eq!(resolved.run.exit, Exit::Code(0), "{}", resolved.run.stderr_text());
    let insts = resolved.inst_lines();
    for name in BOOT_REQUEST {
        assert!(
            insts
                .iter()
                .any(|line| line.strip_prefix(b"Inst ").and_then(|rest| rest.split(|byte| *byte == b' ').next())
                    == Some(*name)),
            "{name:?}"
        );
    }
    assert!(
        resolved.list("main").expect("apt left main's Packages list") == scenario.index(),
        "checked index byte for byte"
    );
    scenario.settle(case, &resolved.recording());
}
#[test]
fn an_unknown_package_fails_naming_it() {
    let (scenario, case) = (Resolve::new(), resolve::case("unknown-package"));
    let resolved = scenario.run(case);
    assert_eq!(resolved.run.exit, Exit::Code(25), "{}", resolved.run.stderr_text());
    assert!(resolved.run.stderr_text().contains("E: Unable to locate package hearth-absent"));
    scenario.settle(case, &resolved.recording());
}
#[test]
fn a_virtual_package_alone_fails_with_no_installation_candidate() {
    let (scenario, case) = (Resolve::new(), resolve::case("no-candidate"));
    let resolved = scenario.run(case);
    assert_eq!(resolved.run.exit, Exit::Code(25), "{}", resolved.run.stderr_text());
    assert!(resolved.run.stderr_text().contains("E: Package 'mail-transport-agent' has no installation candidate"));
    scenario.settle(case, &resolved.recording());
}
#[test]
fn conflicting_requests_fail_after_the_essential_set_naming_neither_mta() {
    let (scenario, case) = (Resolve::new(), resolve::case("unsatisfiable"));
    let resolved = scenario.run(case);
    assert_eq!(resolved.run.exit, Exit::Code(25), "{}", resolved.run.stderr_text());
    assert!(resolved.run.stderr_text().contains("E: Unable to correct problems"));
    let insts = resolved.inst_lines();
    assert!(!insts.is_empty(), "the essential set is simulated first");
    assert!(!insts.iter().any(|line| line.starts_with(b"Inst postfix ") || line.starts_with(b"Inst exim4")));
    scenario.settle(case, &resolved.recording());
}
#[test]
fn an_unreachable_mirror_fails_with_no_closure() {
    let (scenario, case) = (Resolve::new(), resolve::case("unreachable-mirror"));
    let resolved = scenario.run(case);
    assert_eq!(resolved.run.exit, Exit::Code(25), "{}", resolved.run.stderr_text());
    assert!(resolved.inst_lines().is_empty(), "no closure from an unreachable mirror");
    assert!(resolved.run.stderr_text().contains("Could not connect to 127.0.0.1"));
    scenario.settle(case, &resolved.recording());
}
#[test]
fn a_snapshot_the_mirror_lacks_fails_on_its_inrelease_not_found() {
    let (scenario, case) = (Resolve::new(), resolve::case("absent-snapshot"));
    let resolved = scenario.run(case);
    assert_eq!(resolved.run.exit, Exit::Code(25), "{}", resolved.run.stderr_text());
    assert!(resolved.served.iter().any(|answer| answer.status == 404
        && answer.target
            == b"http://snapshot.debian.org/archive/debian/20000101T000000Z/dists/forky/InRelease".as_slice().into()));
    assert!(resolved.inst_lines().is_empty(), "no closure from a missing snapshot");
    scenario.settle(case, &resolved.recording());
}
#[test]
fn several_includes_resolve_as_one() {
    let (scenario, case) = (Resolve::new(), resolve::case("test-boot"));
    let resolved = scenario.run_under(case, &Limits { include: 32, ..limits::SHIPPED });
    scenario.compare(case, &resolved.recording());
}
#[test]
fn a_small_include_limit_splits_the_vector() {
    let scenario = Resolve::new();
    let invocation = vector::simulate(
        &scenario.simulate(resolve::case("test-boot")),
        b"/scratch",
        None,
        &Limits { include: 32, ..limits::SHIPPED },
    )
    .expect("split request");
    assert!(invocation.args.iter().filter(|arg| arg.starts_with(b"--include=")).count() > 1);
}
