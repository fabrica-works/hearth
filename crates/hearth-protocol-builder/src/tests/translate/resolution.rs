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

fn record_back(record: crate::builder::store::vocabulary::Resolution) -> crate::tree::resolution::Resolution {
    use crate::tree::resolution::{Costs, Index, Inputs, Line, Made, Package, Resolution};
    let mut closure = skein_lib::List::with_capacity(u32::try_from(record.closure.len()).expect("closure"));
    for package in record.closure {
        closure
            .push(Package {
                name: package.name,
                architecture: package.architecture,
                version: package.version,
                source: package.source,
                filename: package.filename,
                size: package.size,
                sha256: package.sha256,
            })
            .expect("closure");
    }
    let mut sources = skein_lib::List::with_capacity(u32::try_from(record.sources.len()).expect("sources"));
    for line in record.sources {
        let mut indices = skein_lib::List::with_capacity(u32::try_from(line.indices.len()).expect("indices"));
        for index in line.indices {
            indices.push(Index { path: index.path, size: index.size, sha256: index.sha256 }).expect("indices");
        }
        sources
            .push(Line { uri: line.uri, suite: line.suite, components: line.components, indices: indices.into_boxed() })
            .expect("sources");
    }
    Resolution {
        key: record.key,
        digest: record.digest,
        inputs: Inputs {
            apt: record.inputs.apt,
            request: record.inputs.request,
            sources: record.inputs.sources,
            variant: record.inputs.variant,
        },
        closure: closure.into_boxed(),
        sources: sources.into_boxed(),
        made: Made {
            at: record.made.at,
            apt: record.made.apt,
            mmdebstrap: record.made.mmdebstrap,
            costs: Costs { fetched: record.made.costs.fetched, took: record.made.costs.took },
        },
    }
}
#[test]
fn a_loaded_resolution_translates_whole_to_the_stores() {
    for (row, inputs) in super::super::fixtures::every_inputs().into_iter().enumerate() {
        let input = super::super::fixtures::resolution_record(inputs);
        let back = record_back(crate::translate::store::resolution(input.clone()));
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
