mod fixtures;
use skein_io::kernel::Fd;
#[test]
fn a_second_output_is_refused_and_stays_with_its_caller() {
    let mut service = fixtures::service();
    assert_eq!(service.adopt_output_file(Fd::new(4)), Ok(()), "one client output");
    assert_eq!(service.adopt_output_pipe(Fd::new(5)), Err(Fd::new(5)), "a second descriptor stays with its caller");
}

#[test]
fn a_beneath_request_reaches_io_with_its_roots_descriptor() {
    use crate::protocol::{Asked, Beneath};
    use skein_lib::{Token, bytes};
    let mut service = fixtures::service();
    let root = service.files.adopt_root(Fd::new(47)).expect("directory token");
    service.asked.push(Asked::Beneath(Beneath::OpenDirectory {
        owner: Token::new(20),
        root,
        name: bytes::copy_of(b"child"),
        no_follow: true,
    }));
    crate::io_down(&mut service);
    let submit = service.file_submissions.pop().expect("open directory");
    assert_eq!(
        submit.kind,
        skein_io::kernel::Op::Open {
            root: Fd::new(47),
            path: bytes::copy_of(b"child"),
            how: skein_io::kernel::OpenHow::DirectoryNoFollow
        }
    );
}

#[test]
fn a_beneath_rename_reaches_io_with_both_descriptors() {
    use crate::protocol::{Asked, Beneath};
    use skein_lib::{Token, bytes};
    let mut service = fixtures::service();
    let from_root = service.files.adopt_root(Fd::new(47)).expect("lists token");
    service.asked.push(Asked::Beneath(Beneath::Rename {
        owner: Token::new(20),
        from_root,
        from: bytes::copy_of(b"index"),
        to_root: Fd::new(4),
        to: bytes::copy_of(b"digest"),
    }));
    crate::io_down(&mut service);
    assert_eq!(
        service.file_submissions.pop().expect("rename").kind,
        skein_io::kernel::Op::Rename {
            from_dir: Fd::new(47),
            from: bytes::copy_of(b"index"),
            to_dir: Fd::new(4),
            to: bytes::copy_of(b"digest")
        }
    );
}
