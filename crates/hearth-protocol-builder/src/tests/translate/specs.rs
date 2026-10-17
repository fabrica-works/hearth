use super::{
    boot, client_kind_back, client_meta, config, exports, live, optional_client_firmware_back, packages, records,
    render, secrets, target,
};
pub(super) fn spec_back(spec: records::records::Spec) -> Option<render::parts::Spec> {
    let records::records::Spec { meta, meta_digest, commit, digest, parts, resolution } = spec;
    let mut translated = skein_lib::List::with_capacity(u32::try_from(parts.len()).expect("bounded parts"));
    for records::records::Part { digest, record } in parts {
        let record = match record {
            records::records::PartRecord::Config(value) => config::part_back(value),
            records::records::PartRecord::Secrets(value) => render::parts::PartRecord::Secrets(secrets::back(value)),
            records::records::PartRecord::Boot(value) => boot::part_back(value),
            records::records::PartRecord::Packages(value) => packages::part_back(value),
            records::records::PartRecord::Live(value) => render::parts::PartRecord::Live(live::back(value)),
            records::records::PartRecord::Exports(exports) => {
                render::parts::PartRecord::Exports(exports::exports_back(exports))
            }
        };
        translated.push(render::parts::Part { digest: hearth_lib::encode::Digest(digest), record }).expect("part room");
    }
    match resolution {
        None => {}
        Some(
            records::records::Resolution::Needed { key: _ } | records::records::Resolution::Known { key: _, keys: _ },
        ) => return None, // hearth 24.1
    }
    let client_meta::Meta { target, base, member, environment, placement } = meta;
    let client_meta::Placement { kind, firmware, hardware, on } = placement;
    Some(render::parts::Spec {
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
    })
}
