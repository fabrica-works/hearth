use super::{
    boot, client_kind_back, client_meta, config, exports, live, optional_client_firmware_back, packages, records,
    render, secrets, software, target,
};
pub(super) fn spec_back(spec: records::records::Spec) -> render::parts::Spec {
    let records::records::Spec { meta, meta_digest, commit, digest, parts, resolution } = spec;
    let mut translated = skein_lib::List::with_capacity(u32::try_from(parts.len()).expect("bounded parts"));
    for records::records::Part { digest, record } in parts {
        let record = match record {
            records::records::PartRecord::Data(value) => render::parts::PartRecord::Data(super::data::back(value)),
            records::records::PartRecord::Config(value) => config::part_back(value),
            records::records::PartRecord::Secrets(value) => render::parts::PartRecord::Secrets(secrets::back(value)),
            records::records::PartRecord::Boot(value) => boot::part_back(value),
            records::records::PartRecord::Packages(value) => packages::part_back(value),
            records::records::PartRecord::Software(value) => render::parts::PartRecord::Software(software::back(value)),
            records::records::PartRecord::Live(value) => render::parts::PartRecord::Live(live::back(value)),
            records::records::PartRecord::Exports(exports) => {
                render::parts::PartRecord::Exports(exports::exports_back(exports))
            }
        };
        translated.push(render::parts::Part { digest: hearth_lib::encode::Digest(digest), record }).expect("part room");
    }
    let resolution = optional_resolution_back(resolution);
    let client_meta::Meta { target, base, member, environment, placement } = meta;
    let client_meta::Placement { kind, firmware, hardware, on } = placement;
    render::parts::Spec {
        resolution,
        meta: render::parts::meta::Meta {
            target,
            base,
            member,
            environment,
            placement: render::parts::meta::Placement {
                kind: client_kind_back(kind),
                firmware: optional_client_firmware_back(firmware),
                hardware,
                on,
            },
        },
        meta_digest: hearth_lib::encode::Digest(meta_digest),
        commit: target::Oid(commit),
        digest: hearth_lib::encode::Digest(digest),
        parts: translated.into_boxed(),
    }
}

fn resolution_back(value: records::records::Resolution) -> render::keys::Resolution {
    match value {
        records::records::Resolution::Needed { key } => {
            render::keys::Resolution::Needed { key: hearth_lib::encode::Digest(key) }
        }
        records::records::Resolution::Known { key, keys } => render::keys::Resolution::Known {
            key: hearth_lib::encode::Digest(key),
            keys: render::keys::Keys {
                packages: hearth_lib::encode::Digest(keys.packages),
                software: hearth_lib::encode::Digest(keys.software),
                config: hearth_lib::encode::Digest(keys.config),
            },
        },
    }
}

fn optional_resolution_back(value: Option<records::records::Resolution>) -> Option<Box<render::keys::Resolution>> {
    Some(Box::new(resolution_back(value?)))
}
