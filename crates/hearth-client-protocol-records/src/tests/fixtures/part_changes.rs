use crate::{
    diff::PartChange,
    parts::{live, packages, secrets, software},
    records::Record,
};
use skein_lib::bytes::copy_of;
#[derive(Clone, Copy, Debug)]
pub enum ChangedPart {
    Secrets,
    Software,
    Live,
    Seeds,
}
pub fn part_change(part: ChangedPart) -> Record {
    let part = match part {
        ChangedPart::Secrets => PartChange::Secrets(secrets::Change {
            files: Box::new([
                secrets::Named::Added(copy_of(b"new")),
                secrets::Named::Removed(copy_of(b"gone")),
                secrets::Named::Changed(copy_of(b"key")),
            ]),
            passwords: Box::new([secrets::Named::Changed(copy_of(b"free"))]),
            identity: Box::new([secrets::Named::Changed(copy_of(b"ssh"))]),
        }),
        ChangedPart::Software => PartChange::Software(software::Change {
            downloads: Box::new([software::DownloadKeyed::Changed {
                key: copy_of(b"/opt/tool"),
                before: download([1; 32]),
                after: download([2; 32]),
            }]),
            scripts: None,
        }),
        ChangedPart::Live => PartChange::Live(live::Change {
            paths: Box::new([live::ConsumersKeyed::Changed {
                key: copy_of(b"/etc/mailname"),
                before: Box::new([]),
                after: Box::new([live::Consumer { task: copy_of(b"mail"), unit: None, action: live::Action::None }]),
            }]),
        }),
        ChangedPart::Seeds => {
            let PartChange::Packages(mut packages) = super::unchanged_packages_change() else { panic!("packages") };
            packages.seeds = packages::SeedsChange {
                users: Box::new([packages::SeedUserKeyed::Changed {
                    key: copy_of(b"mail"),
                    before: packages::SeedIds { uid: 120, gid: None },
                    after: packages::SeedIds { uid: 121, gid: Some(122) },
                }]),
                groups: Box::new([packages::SeedGroupKeyed::Changed {
                    key: copy_of(b"mail"),
                    before: 120,
                    after: 122,
                }]),
            };
            PartChange::Packages(packages)
        }
    };
    super::change_record(part)
}
fn download(sha256: [u8; 32]) -> software::Download {
    software::Download {
        path: copy_of(b"/opt/tool"),
        sha256,
        owner: copy_of(b"root"),
        group: copy_of(b"root"),
        how: software::How::Tar { strip: 1 },
    }
}
pub fn diff_json(record: &Record) -> Result<Box<[u8]>, crate::form::Refusal> {
    crate::form::print(
        crate::form::Form::JsonLines,
        record,
        Some(crate::records::Type::Change),
        &crate::limits::SHIPPED,
    )
}
