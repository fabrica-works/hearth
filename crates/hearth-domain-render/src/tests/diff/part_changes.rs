use crate::{
    diff::{self, Layer},
    parts::{live, packages, secrets, software},
    tests::fixtures,
};
use hearth_lib::encode::Digest;
use skein_lib::bytes::copy_of;
#[test]
fn a_changed_download_is_keyed_by_path() {
    let (before, after) = fixtures::changed_download();
    assert_eq!(
        software::change(&before, &after),
        software::Change {
            downloads: Box::new([software::DownloadKeyed::Changed {
                key: copy_of(b"/opt/tool"),
                before: software::Download {
                    path: copy_of(b"/opt/tool"),
                    sha256: fixtures::ABC_SHA256,
                    owner: copy_of(b"root"),
                    group: copy_of(b"root"),
                    how: software::How::File { mode: 0o755 }
                },
                after: software::Download {
                    path: copy_of(b"/opt/tool"),
                    sha256: Digest([2; 32]),
                    owner: copy_of(b"root"),
                    group: copy_of(b"root"),
                    how: software::How::File { mode: 0o755 }
                },
            }]),
            scripts: None,
        }
    );
}
#[test]
fn secrets_change_by_name_only() {
    let (before, after) = fixtures::changed_secrets();
    assert_eq!(
        secrets::change(&before, &after),
        secrets::Change {
            files: Box::new([secrets::Named::Changed(copy_of(b"key"))]),
            passwords: Box::new([secrets::Named::Changed(copy_of(b"free"))]),
            identity: Box::new([secrets::Named::Changed(copy_of(b"ssh"))]),
        }
    );
}
#[test]
fn a_live_path_gaining_a_consumer_is_changed() {
    let (before, after) = fixtures::changed_live();
    assert_eq!(
        live::change(&before, &after),
        live::Change {
            paths: Box::new([live::ConsumersKeyed::Changed {
                key: copy_of(b"/etc/mailname"),
                before: Box::new([live::Consumer {
                    task: copy_of(b"motd"),
                    unit: Some(copy_of(b"postfix.service")),
                    action: live::Action::Reload
                }]),
                after: Box::new([
                    live::Consumer {
                        task: copy_of(b"motd"),
                        unit: Some(copy_of(b"postfix.service")),
                        action: live::Action::Reload
                    },
                    live::Consumer { task: copy_of(b"other"), unit: None, action: live::Action::None },
                ]),
            }])
        }
    );
}
#[test]
fn seeds_change_as_users_and_groups_added_removed_changed_by_name() {
    use packages::{SeedIds, SeedUserKeyed};
    let (before, after) = fixtures::changed_seeds();
    assert_eq!(
        packages::change(&before, &after).seeds.users.as_ref(),
        &[
            SeedUserKeyed::Added { key: copy_of(b"added"), value: SeedIds { uid: 122, gid: Some(123) } },
            SeedUserKeyed::Changed {
                key: copy_of(b"changed"),
                before: SeedIds { uid: 120, gid: None },
                after: SeedIds { uid: 121, gid: Some(121) }
            },
            SeedUserKeyed::Removed { key: copy_of(b"removed"), value: SeedIds { uid: 120, gid: Some(121) } },
        ]
    );
}
#[test]
fn seed_groups_change_by_name_in_byte_order() {
    use packages::SeedGroupKeyed;
    let (before, after) = fixtures::changed_seeds();
    assert_eq!(
        packages::change(&before, &after).seeds.groups.as_ref(),
        &[
            SeedGroupKeyed::Added { key: copy_of(b"added"), value: 122 },
            SeedGroupKeyed::Changed { key: copy_of(b"changed"), before: 120, after: 121 },
            SeedGroupKeyed::Removed { key: copy_of(b"removed"), value: 120 },
        ]
    );
}
#[test]
fn the_three_layers_are_slot_software_other() {
    use crate::parts::PartRecord;
    let (secrets, _) = fixtures::changed_secrets();
    let (software, _) = fixtures::changed_download();
    let (live, _) = fixtures::changed_live();
    assert_eq!(
        [
            diff::layer(&PartRecord::Secrets(secrets)),
            diff::layer(&PartRecord::Software(software)),
            diff::layer(&PartRecord::Live(live))
        ],
        [Layer::Slot, Layer::Software, Layer::Other]
    );
}

#[test]
fn a_fetch_hint_changed_alone_is_no_software_change() {
    assert_eq!(
        software::change(
            &fixtures::mirrored(b"https://a.example/tool"),
            &fixtures::mirrored(b"https://b.example/tool")
        ),
        software::Change { downloads: Box::new([]), scripts: None }
    );
}
