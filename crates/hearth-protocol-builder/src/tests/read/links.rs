use super::super::fixtures::{self, Layer, REQUEST};
use crate::{
    builder::vocabulary::Event as RootEvent,
    render::vocabulary::{As, Decoded, Event},
};
use skein_io::kernel::Exit;
#[test]
fn a_tree_link_reaches_render_as_its_target() {
    let mut layer = Layer::new();
    layer.read(Box::new([crate::render::vocabulary::Blob { kind: As::Link, ..fixtures::blob(b"/proc/self/mounts") }]));
    assert_eq!(
        layer.run(&fixtures::batch(b"/proc/self/mounts"), Exit::Code(0)),
        RootEvent::Render(Event::Read {
            request: REQUEST,
            oid: fixtures::oid(),
            read: Decoded::Link(b"/proc/self/mounts".as_slice().into())
        })
    );
}
