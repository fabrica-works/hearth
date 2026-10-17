use super::fixtures::{self, INPUT, Layer, REQUEST};
use crate::{
    Asked,
    builder::{
        render::{
            revision::{Document, Firmware, Layer as DocumentLayer, Placement, Position, Runs},
            vocabulary::{Decoded, Event},
        },
        vocabulary::Event as RootEvent,
    },
};
use skein_io::{self as io, kernel::Exit};
use skein_lib::{
    List, bytes,
    stream::{Down, Held},
};

#[test]
fn a_read_sends_each_request_line_then_ends_standard_input() {
    let mut layer = Layer::new();
    layer.read(Box::new([
        fixtures::blob(b"{}"),
        fixtures::blob_named(b"1123456789abcdef0123456789abcdef01234567", b"{}"),
    ]));
    let mut held = Held::new(fixtures::batch(b"{}"));
    let mut sent = List::with_capacity(3);
    for _ in 0_u32..32 {
        let request = layer.asked().expect("stdin work");
        if let Asked::Io(io::Request::Stream { stream: INPUT, down: Down::Send(bytes) }) = &request {
            sent.push(Some(bytes.clone())).expect("two lines");
        }
        if request == Asked::Io(io::Request::Stream { stream: INPUT, down: Down::Finish }) {
            sent.push(None).expect("one finish");
            break;
        }
        layer.answer(request, &mut held, Exit::Code(0));
    }
    assert_eq!(
        sent.into_boxed().as_ref(),
        &[
            Some(bytes::copy_of(b"0123456789abcdef0123456789abcdef01234567\n")),
            Some(bytes::copy_of(b"1123456789abcdef0123456789abcdef01234567\n")),
            None
        ],
        "one line per blob, then Finish"
    );
}
#[test]
fn a_documents_blob_reaches_render_decoded_with_its_placement() {
    let mut layer = Layer::new();
    layer.read(Box::new([fixtures::blob(fixtures::DOCUMENT)]));
    assert_eq!(
        layer.run(&fixtures::batch(fixtures::DOCUMENT), Exit::Code(0)),
        RootEvent::Render(Event::Read {
            request: REQUEST,
            oid: crate::render::revision::Oid(bytes::copy_of(b"0123456789abcdef0123456789abcdef01234567")),
            read: Decoded::Document(Document {
                at: Position { line: 1, column: 1 },
                layer: DocumentLayer {
                    tasks: Box::new([]),
                    placement: Some(Placement {
                        at: Position { line: 1, column: 14 },
                        kind: Runs::Metal,
                        firmware: Some(Firmware::Efi),
                        hardware: Some(bytes::copy_of(b"7th-gen")),
                        on: None,
                        mutable: false
                    })
                }
            })
        }),
        "batch pieces become the placed document"
    );
}
#[test]
fn a_document_config_refuses_answers_refused_at_its_position() {
    let mut layer = Layer::new();
    layer.read(Box::new([fixtures::blob(b"{\"placement\":{}}")]));
    assert_eq!(
        layer.run(&fixtures::batch(b"{\"placement\":{}}"), Exit::Code(0)),
        RootEvent::Render(Event::Read {
            request: REQUEST,
            oid: crate::render::revision::Oid(bytes::copy_of(b"0123456789abcdef0123456789abcdef01234567")),
            read: Decoded::Refused { at: Position { line: 1, column: 14 } }
        }),
        "configuration refusal retains its position"
    );
}
const BAD_BATCH: &[&[u8]] = &[
    b"0123456789abcdef0123456789abcdef01234567 missing\n",
    b"0123456789abcdef0123456789abcdef01234567 blob 3\n{}\n",
    b"garbled\n",
];
#[test]
fn a_batch_machine_failure_answers_operational() {
    for stdout in BAD_BATCH {
        let mut layer = Layer::new();
        layer.read(Box::new([fixtures::blob(b"{}")]));
        assert_eq!(
            layer.run(stdout, Exit::Code(0)),
            RootEvent::Render(Event::Failed { request: REQUEST, class: crate::render::vocabulary::Class::Operational }),
            "batch {stdout:?}"
        );
    }
}

#[test]
fn the_last_document_waits_for_successful_exit_and_resource_closure() {
    let mut layer = Layer::new();
    layer.read(Box::new([fixtures::blob(b"{}")]));
    let Some(Asked::Io(io::Request::Spawn { owner, .. })) = layer.asked() else {
        panic!("spawn");
    };
    layer.spawn_pending(owner, true);
    let mut held = Held::new(fixtures::batch(b"{}"));
    for _ in 0_u32..32 {
        let Some(request) = layer.asked() else {
            break;
        };
        layer.answer(request, &mut held, Exit::Code(0));
    }
    assert_eq!(layer.told(), None, "the decoded last blob waits for exit");
    layer.up(io::Event::Exited { owner, exit: Exit::Code(0) });
    assert_eq!(
        layer.run(b"", Exit::Code(0)),
        RootEvent::Render(Event::Read {
            request: REQUEST,
            oid: crate::render::revision::Oid(bytes::copy_of(b"0123456789abcdef0123456789abcdef01234567")),
            read: Decoded::Document(Document {
                at: Position { line: 1, column: 1 },
                layer: DocumentLayer { tasks: Box::new([]), placement: None }
            })
        }),
        "the decoded blob follows Closed for every resource"
    );
}
#[test]
fn a_batch_failure_closes_the_child_without_waiting_for_exit() {
    let mut layer = Layer::new();
    layer.read(Box::new([fixtures::blob(b"{}")]));
    let Some(Asked::Io(io::Request::Spawn { owner, .. })) = layer.asked() else {
        panic!("spawn");
    };
    layer.spawn_pending(owner, true);
    assert_eq!(
        layer.run(b"garbled\n", Exit::Code(0)),
        RootEvent::Render(Event::Failed { request: REQUEST, class: crate::render::vocabulary::Class::Operational }),
        "a failed batch closes immediately"
    );
}
