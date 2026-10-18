use crate::{Configuration, Roots, Service, domain::vocabulary::Call, limits, protocol::Form};
use alloc::boxed::Box;
use skein_io::kernel::Fd;
use skein_lib::{Token, bytes};
pub const UID: u32 = 1_000;
pub const SERVICE_SEED: u64 = 7;
pub const OWNER: Token = Token::new(20);
pub const SOURCE_ROOT: Fd = Fd::new(47);
pub const OUTPUT: Fd = Fd::new(4);
pub const OTHER_OUTPUT: Fd = Fd::new(5);
pub fn service() -> Service {
    Service::new(
        &Configuration { proxy: None, store: bytes::copy_of(b"/store"), limits: limits::SHIPPED, uid: UID },
        Roots { store: Fd::new(3), cache: Fd::new(4), resolutions: Fd::new(5), staging: Fd::new(6) },
        Call::Render {
            resolve: false,
            repository: bytes::copy_of(b"/repo"),
            revision: bytes::copy_of(b"HEAD"),
            targets: Box::new([]),
        },
        Form::Text,
        SERVICE_SEED,
    )
}

pub const LARGER_QUEUE: u32 = limits::SHIPPED.queue + 1;
/// Render's shipped builder budget (render.md, 8.4).
pub const BUILDER_BYTES: u64 = 1 << 30;

pub fn with_source_root() -> (Service, Token) {
    let mut service = service();
    let root = service.files.adopt_root(SOURCE_ROOT).expect("directory token");
    (service, root)
}
