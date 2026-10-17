use crate::{
    render::{self, parts::packages as target},
    tests::fixtures,
    translate::resolution,
};
use hearth_protocol_mmdebstrap::{limits, vector};
fn simulate_back(input: vector::Simulate) -> target::Packages {
    let mut packages = fixtures::apt_packages();
    let source = input.sources;
    packages.sources = target::Sources {
        suite: source.suite.as_bytes().into(),
        architecture: source.architecture.as_bytes().into(),
        components: components_back(source.components),
        archive: input_archive(&source.archive),
        updates: source.updates,
        security: optional_archive(source.security),
    };
    let mut names = skein_lib::List::with_capacity(u32::try_from(input.request.len()).expect("request bound"));
    for name in input.request {
        names.push(name.as_bytes().into()).expect("name capacity");
    }
    packages.request = names.into_boxed();
    packages.variant = match input.variant {
        vector::Variant::Apt => target::Variant::Apt,
    };
    packages.apt = target::Apt { install_recommends: input.apt.install_recommends };
    packages
}
fn input_archive(time: &vector::Timestamp) -> Box<[u8]> {
    // Timestamp intentionally hides its bytes; its source line is the public representation.
    let sources = vector::Sources {
        suite: vector::Name::new(b"forky", &limits::SHIPPED).expect("suite"),
        architecture: vector::Name::new(b"amd64", &limits::SHIPPED).expect("arch"),
        components: Box::new([vector::Component::Main]),
        archive: time.clone(),
        updates: false,
        security: None,
    };
    let lines = vector::lines(&sources).expect("source lines");
    fixtures::only(&lines)
        .uri
        .strip_prefix(vector::ARCHIVE)
        .expect("archive URI")
        .strip_suffix(b"/")
        .expect("slash")
        .into()
}
#[test]
fn sources_names_variant_and_apt_translate_to_simulate_and_back() {
    for (row, mut input) in fixtures::package_records().into_iter().enumerate() {
        input.apt.install_recommends = input.variant == target::Variant::Container;
        input.variant = target::Variant::Apt;
        let simulated =
            resolution::simulate(&input.sources, &input.request, input.variant, &input.apt, &limits::SHIPPED)
                .expect("simulate");
        let back = simulate_back(simulated);
        let input = (input.sources, input.request, input.variant, input.apt);
        let back = (back.sources, back.request, back.variant, back.apt);
        assert_eq!(back, input, "row {row}: resolution inputs through simulate and back");
    }
}
#[test]
fn a_resolve_of_test_min_translates_to_41_1s_simulate() {
    let render::vocabulary::Request::Resolve { mut sources, names, variant, apt, .. } =
        fixtures::resolve(fixtures::KEY, &[])
    else {
        panic!("resolve")
    };
    sources.archive = b"20261017T143929Z".as_slice().into();
    let simulated = resolution::simulate(&sources, &names, variant, &apt, &limits::SHIPPED).expect("simulate");
    assert_eq!(
        vector::sources_list(&vector::lines(&simulated.sources).expect("sources")).as_ref(),
        b"deb http://snapshot.debian.org/archive/debian/20261017T143929Z/ forky main\n"
    );
    assert_eq!(simulated.request.as_ref(), []);
}

fn optional_archive(time: Option<vector::Timestamp>) -> Option<Box<[u8]>> {
    Some(input_archive(&time?))
}
fn components_back(components: Box<[vector::Component]>) -> Box<[target::Component]> {
    let mut back = skein_lib::List::with_capacity(4);
    for component in components {
        back.push(match component {
            vector::Component::Main => target::Component::Main,
            vector::Component::Contrib => target::Component::Contrib,
            vector::Component::NonFree => target::Component::NonFree,
            vector::Component::NonFreeFirmware => target::Component::NonFreeFirmware,
        })
        .expect("four components");
    }
    back.into_boxed()
}
fn inst_back(inst: render::resolution::Inst) -> hearth_protocol_mmdebstrap::output::Inst {
    hearth_protocol_mmdebstrap::output::Inst {
        name: fixtures::mmdebstrap_name(&inst.name),
        version: inst.version,
        architecture: fixtures::mmdebstrap_name(&inst.architecture),
    }
}
#[test]
fn install_triples_translate_and_back() {
    for (row, input) in fixtures::insts().into_iter().enumerate() {
        let back = inst_back(resolution::inst(input.clone()));
        assert_eq!(back, input, "row {row}: install triple through render and back");
    }
}
fn cause_back(cause: render::resolution::Cause) -> hearth_protocol_mmdebstrap::tail::Cause {
    use hearth_protocol_mmdebstrap::tail::Cause;
    match cause {
        render::resolution::Cause::Fetch => Cause::Fetch,
        render::resolution::Cause::Resolve => Cause::Resolve,
        render::resolution::Cause::Space => Cause::Space,
        render::resolution::Cause::Absent { .. } => Cause::Absent,
        render::resolution::Cause::Unknown(name) => Cause::Unknown(fixtures::mmdebstrap_name(&name)),
        render::resolution::Cause::Unsatisfiable(name) => Cause::Unsatisfiable(optional_name_back(name)),
        render::resolution::Cause::Other => Cause::Other,
    }
}
fn optional_name_back(name: Option<Box<[u8]>>) -> Option<vector::Name> {
    Some(fixtures::mmdebstrap_name(&name?))
}
#[test]
fn failure_causes_translate_and_back() {
    for (row, input) in fixtures::causes().into_iter().enumerate() {
        let back = cause_back(resolution::cause(input.clone(), &fixtures::source_lines(), &[]));
        assert_eq!(back, input, "row {row}: cause through render and back");
    }
}
