use crate::tests::fixtures;
const FILES: &[Option<hearth_lib::encode::Digest>] = &[None, Some(hearth_lib::encode::Digest([7; 32]))];
#[test]
fn the_initramfs_file_digest_translates_to_the_records_and_back() {
    for (row, &file) in FILES.iter().enumerate() {
        let mut input = fixtures::apt_packages();
        input.initramfs_file = file;
        let translated = crate::translate::parts::packages::packages(input.clone());
        assert_eq!(fixtures::packages_back(translated), input, "row {row}");
    }
}

#[test]
fn a_render_made_file_is_kept_under_its_destination() {
    use crate::builder::vocabulary::Event;
    let (_, _, event) = fixtures::produced_keep_at(b"/etc/subuid");
    let Event::Produced { path, .. } = event else { panic!("produced") };
    assert_eq!(path.as_ref(), b"/etc/subuid");
}
