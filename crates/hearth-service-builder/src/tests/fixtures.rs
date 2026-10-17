use crate::{Configuration, Roots, Service, domain::vocabulary::Call, limits, protocol::Form};
use alloc::boxed::Box;
use skein_io::kernel::Fd;
use skein_lib::bytes;
pub fn service() -> Service {
    Service::new(
        &Configuration { limits: limits::SHIPPED },
        Roots { store: Fd::new(3), cache: Fd::new(4) },
        Call::Render { repository: bytes::copy_of(b"/repo"), revision: bytes::copy_of(b"HEAD"), targets: Box::new([]) },
        Form::Text,
        7,
    )
}
