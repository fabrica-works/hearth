mod fixtures;
use skein_io::kernel::Fd;
#[test]
fn a_second_output_is_refused_and_stays_with_its_caller() {
    let mut service = fixtures::service();
    assert_eq!(service.adopt_output_file(Fd::new(4)), Ok(()), "one client output");
    assert_eq!(service.adopt_output_pipe(Fd::new(5)), Err(Fd::new(5)), "a second descriptor stays with its caller");
}
