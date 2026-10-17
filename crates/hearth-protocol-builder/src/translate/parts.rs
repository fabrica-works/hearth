//! Render's part vocabulary mapped to client records (render.md, 4.2).
use crate::{records::records as target, render::parts as source};
pub mod boot;
pub mod config;
pub mod data;
pub mod exports;
pub mod live;
pub mod packages;
pub mod secrets;
pub mod software;
#[must_use]
pub fn part(part: source::Part) -> target::Part {
    let source::Part { digest, record } = part;
    target::Part {
        digest: digest.0,
        record: match record {
            source::PartRecord::Data(value) => target::PartRecord::Data(data::data(value)),
            source::PartRecord::Config(value) => target::PartRecord::Config(config::config(value)),
            source::PartRecord::Secrets(value) => target::PartRecord::Secrets(secrets::secrets(value)),
            source::PartRecord::Packages(value) => target::PartRecord::Packages(packages::packages(value)),
            source::PartRecord::Software(value) => target::PartRecord::Software(software::software(value)),
            source::PartRecord::Live(value) => target::PartRecord::Live(live::live(value)),
            source::PartRecord::Boot(value) => target::PartRecord::Boot(boot::boot(value)),
            source::PartRecord::Exports(value) => target::PartRecord::Exports(exports::exports(value)),
        },
    }
}
