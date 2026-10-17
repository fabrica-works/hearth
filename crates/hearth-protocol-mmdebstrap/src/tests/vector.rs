use super::fixtures;
use crate::{
    limits,
    vector::{self, Component, Name, Proxy, Timestamp, VectorError},
};
use alloc::boxed::Box;
const SCRATCH: &[u8] = b"/store/staging/k";
#[test]
fn the_vector_is_render_s_for_a_forky_main_request() {
    let simulate = fixtures::simulate(&[b"apt", b"mmdebstrap"]);
    let made = vector::simulate(&simulate, SCRATCH, None, &limits::SHIPPED).unwrap();
    let want: [&[u8]; 12] = [
        b"--verbose",
        b"--simulate",
        b"--mode=chrootless",
        b"--variant=apt",
        b"--architectures=amd64",
        b"--include=apt,mmdebstrap",
        b"--aptopt=Acquire::Check-Valid-Until \"false\"",
        b"--aptopt=APT::Install-Recommends \"false\"",
        b"--skip=cleanup/apt/lists",
        b"forky",
        b"/store/staging/k/root",
        b"/store/staging/k/sources.list",
    ];
    assert_eq!(made.args.len(), want.len());
    for (arg, w) in made.args.iter().zip(want) {
        assert_eq!(&**arg, w);
    }
    assert_eq!(&*made.sources_list, b"deb http://snapshot.debian.org/archive/debian/20261001T030405Z/ forky main\n");
    assert_eq!(vector::PROGRAM, b"/usr/bin/mmdebstrap");
}
#[test]
fn the_environment_is_given_whole_with_tmpdir_the_scratch() {
    let input = fixtures::simulate(&[]);
    let made = vector::simulate(&input, SCRATCH, None, &limits::SHIPPED).unwrap();
    let want: [&[u8]; 3] = [b"LC_ALL=C", b"PATH=/usr/sbin:/usr/bin:/sbin:/bin", b"TMPDIR=/store/staging/k"];
    assert_eq!(made.env.len(), 3);
    for (arg, want) in made.env.iter().zip(want) {
        assert_eq!(&**arg, want);
    }
}
#[test]
fn both_pockets_add_updates_and_security_lines_after_the_archive_s() {
    let mut input = fixtures::simulate(&[]);
    input.sources.updates = true;
    input.sources.security = Timestamp::new(b"20260930T000000Z");
    input.sources.components = Box::from([Component::Main, Component::Contrib, Component::NonFreeFirmware]);
    let made = vector::simulate(&input, SCRATCH, None, &limits::SHIPPED).unwrap();
    assert_eq!(&*made.sources_list, b"deb http://snapshot.debian.org/archive/debian/20261001T030405Z/ forky main contrib non-free-firmware\ndeb http://snapshot.debian.org/archive/debian/20261001T030405Z/ forky-updates main contrib non-free-firmware\ndeb http://snapshot.debian.org/archive/debian-security/20260930T000000Z/ forky-security main contrib non-free-firmware\n");
}
#[test]
fn a_proxy_adds_its_apt_option_before_the_skip() {
    let input = fixtures::simulate(&[]);
    let proxy = Proxy::new(b"http://127.0.0.1:8080", &limits::SHIPPED).unwrap();
    let made = vector::simulate(&input, SCRATCH, Some(&proxy), &limits::SHIPPED).unwrap();
    assert_eq!(&*made.args[7], b"--aptopt=Acquire::http::Proxy \"http://127.0.0.1:8080\"");
    assert_eq!(&*made.args[8], b"--skip=cleanup/apt/lists");
}
#[test]
fn install_recommends_is_passed_as_apt_s_option() {
    let mut input = fixtures::simulate(&[]);
    input.apt.install_recommends = true;
    let made = vector::simulate(&input, SCRATCH, None, &limits::SHIPPED).unwrap();
    assert_eq!(&*made.args[6], b"--aptopt=APT::Install-Recommends \"true\"");
}
#[test]
fn a_request_past_one_argument_is_split_into_several_includes() {
    let input = fixtures::simulate(&[b"aaaaaaaaaa", b"bbbbbbbbbb", b"cccccccccc"]);
    let limits = limits::Limits { include: 32, ..limits::SHIPPED }; // exercise splitting without 64 KiB
    let made = vector::simulate(&input, SCRATCH, None, &limits).unwrap();
    assert_eq!(&*made.args[5], b"--include=aaaaaaaaaa,bbbbbbbbbb");
    assert_eq!(&*made.args[6], b"--include=cccccccccc");
}
#[test]
fn an_empty_request_passes_no_include() {
    let made = vector::simulate(&fixtures::simulate(&[]), SCRATCH, None, &limits::SHIPPED).unwrap();
    assert_eq!(made.args.len(), 11);
    for arg in made.args {
        assert!(!arg.starts_with(b"--include="), "empty request has no include");
    }
}
#[test]
fn a_request_out_of_order_or_components_without_main_are_refused() {
    for names in [[b"bb".as_slice(), b"aa".as_slice()], [b"aa", b"aa"]] {
        assert_eq!(
            vector::simulate(&fixtures::simulate(&names), SCRATCH, None, &limits::SHIPPED),
            Err(VectorError::Request)
        );
    }
    let cases: &[&[Component]] = &[
        &[],
        &[Component::Contrib],
        &[Component::Main, Component::Main],
        &[Component::Main, Component::NonFree, Component::Contrib],
    ];
    for components in cases {
        let mut input = fixtures::simulate(&[]);
        input.sources.components = Box::from(*components);
        assert_eq!(vector::lines(&input.sources), Err(VectorError::Components));
    }
}
const NAMES: &[&[u8]] = &[b"a", b".aa", b"Aa", b"aa bb", b"aa\0bb"];
const TIMES: &[&[u8]] = &[b"20261001T03040Z", b"20261001x030405Z", b"20261001Ta30405Z"];
const PROXIES: &[&[u8]] =
    &[b"https://host", b"http://a\"", b"http://a;", b"http://a\\", b"http://a{", b"http://a}", b"http://a\n"];
const SCRATCHES: &[&[u8]] = &[b"relative", b"/trailing/", b"/nul\0path"];
#[test]
fn names_refuse_bytes_outside_policy() {
    for bytes in NAMES {
        assert_eq!(Name::new(bytes, &limits::SHIPPED), None);
    }
}
#[test]
fn timestamps_refuse_bytes_outside_their_form() {
    for bytes in TIMES {
        assert_eq!(Timestamp::new(bytes), None);
    }
}
#[test]
fn proxies_refuse_bytes_outside_their_form() {
    for bytes in PROXIES {
        assert_eq!(Proxy::new(bytes, &limits::SHIPPED), None);
    }
}
#[test]
fn scratch_refuses_bytes_outside_its_form() {
    let input = fixtures::simulate(&[]);
    for bytes in SCRATCHES {
        assert_eq!(vector::simulate(&input, bytes, None, &limits::SHIPPED), Err(VectorError::Scratch));
    }
    let limits = limits::Limits { path: 13, ..limits::SHIPPED }; // suffix cannot fit
    assert_eq!(vector::simulate(&input, b"/a", None, &limits), Err(VectorError::Scratch));
}
