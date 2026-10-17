//! Render's part vocabulary mapped to client records (render.md, 4.2).
use crate::{records::records as target, render::parts as source};
pub mod config;
pub mod packages;
pub mod secrets;
#[must_use]
pub fn part(part: source::Part) -> target::Part {
    let source::Part { digest, record } = part;
    target::Part {
        digest: digest.0,
        record: match record {
            source::PartRecord::Config(value) => target::PartRecord::Config(config::config(value)),
            source::PartRecord::Secrets(value) => target::PartRecord::Secrets(secrets::secrets(value)),
            source::PartRecord::Packages(value) => target::PartRecord::Packages(packages::packages(value)),
        },
    }
}
