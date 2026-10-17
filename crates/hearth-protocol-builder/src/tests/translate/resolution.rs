use super::specs::spec_back;
use crate::translate::answers;
#[test]
fn a_spec_translates_to_the_records_and_back() {
    for (row, input) in super::super::fixtures::resolution_specs()
        .into_iter()
        .chain(super::super::fixtures::live_specs())
        .chain(super::super::fixtures::export_specs())
        .enumerate()
    {
        let back = spec_back(answers::spec(input.clone()));
        assert_eq!(back, input, "row {row}: spec and resolution fields");
    }
}

#[test]
fn a_loaded_resolution_translates_whole_to_the_stores() {
    for (row, inputs) in super::super::fixtures::every_inputs().into_iter().enumerate() {
        let input = super::super::fixtures::resolution_record(inputs);
        let back = crate::translate::store::tree_resolution(crate::translate::store::resolution(input.clone()));
        assert_eq!(back, input, "row {row}: sources, closure and costs");
    }
}

#[test]
fn package_stanzas_translate_and_back() {
    use hearth_protocol_mmdebstrap::packages::Stanza;
    for (row, input) in crate::tests::fixtures::package_stanzas().into_iter().enumerate() {
        let stanza = crate::translate::resolution::stanza(input.clone());
        let back = Stanza {
            package: crate::tests::fixtures::mmdebstrap_name(&stanza.package),
            version: stanza.version,
            architecture: crate::tests::fixtures::mmdebstrap_name(&stanza.architecture),
            filename: stanza.filename,
            size: stanza.size,
            sha256: stanza.sha256.0,
        };
        assert_eq!(back, input, "row {row}: package stanza");
    }
}

#[test]
fn a_resolved_line_holds_the_sources_uri_suite_components_and_indices() {
    use crate::render::resolution::{Index, Line};
    let input = crate::tests::fixtures::index_sources();
    let indices =
        Box::new([Index { path: b"InRelease".as_slice().into(), size: 3, sha256: crate::tests::fixtures::ABC }]);
    assert_eq!(
        crate::translate::resolution::line(crate::tests::fixtures::only(&input), indices),
        Line {
            uri: b"http://one/".as_slice().into(),
            suite: b"forky".as_slice().into(),
            components: Box::new([b"main".as_slice().into()]),
            indices: Box::new([Index {
                path: b"InRelease".as_slice().into(),
                size: 3,
                sha256: crate::tests::fixtures::ABC
            }]),
        }
    );
}
#[test]
fn source_lines_translate_and_back() {
    use hearth_protocol_mmdebstrap::vector::{Component, Line};
    for (row, input) in crate::tests::fixtures::source_lines().into_iter().enumerate() {
        let line = crate::translate::resolution::line(&input, Box::new([]));
        let mut components = skein_lib::List::with_capacity(4);
        for component in line.components {
            let component = match component.as_ref() {
                b"main" => Component::Main,
                b"contrib" => Component::Contrib,
                b"non-free" => Component::NonFree,
                b"non-free-firmware" => Component::NonFreeFirmware,
                component => panic!("component: {component:?}"),
            };
            components.push(component).expect("components");
        }
        let back = Line { uri: line.uri, suite: line.suite, components: components.into_boxed() };
        assert_eq!(back, input, "row {row}: source line");
    }
}

#[test]
fn each_resolution_field_translates_and_back() {
    for (row, inputs) in crate::tests::fixtures::every_inputs().into_iter().enumerate() {
        let input = crate::translate::store::resolution(crate::tests::fixtures::resolution_record(inputs));
        let back = crate::translate::store::resolution(crate::translate::store::tree_resolution(input.clone()));
        assert_eq!(back, input, "row {row}: resolution fields");
    }
}
