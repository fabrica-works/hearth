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
                members: None,
                at: Position { line: 1, column: 1 },
                layer: DocumentLayer {
                    debian: None,
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
            read: Decoded::Refused {
                at: Position { line: 1, column: 14 },
                fault: crate::render::errors::Decode::Missing { field: crate::render::errors::Field::Kind },
                also: None
            }
        }),
        "configuration refusal retains its position"
    );
}
const BAD_BATCH: &[(&[u8], crate::render::errors::Ended)] = &[
    (b"0123456789abcdef0123456789abcdef01234567 blob 3\n{}\n", crate::render::errors::Ended::Output),
    (b"garbled\n", crate::render::errors::Ended::Output),
];
#[test]
fn a_batch_machine_failure_answers_operational() {
    for (stdout, ended) in BAD_BATCH {
        let mut layer = Layer::new();
        layer.read(Box::new([fixtures::blob(b"{}")]));
        assert_eq!(
            layer.run(stdout, Exit::Code(0)),
            RootEvent::Render(Event::Failed {
                request: REQUEST,
                class: crate::render::vocabulary::Class::Operational,
                at: None,
                failure: crate::render::vocabulary::Failure::Tool(crate::render::errors::Tool {
                    command: crate::render::errors::Command::CatFile,
                    ended: ended.clone(),
                    tail: Box::new([])
                })
            }),
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
                members: None,
                at: Position { line: 1, column: 1 },
                layer: DocumentLayer { debian: None, tasks: Box::new([]), placement: None }
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
    layer.spawned(owner, true, Exit::Code(0));
    assert_eq!(
        layer.run(b"garbled\n", Exit::Code(0)),
        RootEvent::Render(Event::Failed {
            request: REQUEST,
            class: crate::render::vocabulary::Class::Operational,
            at: None,
            failure: crate::render::vocabulary::Failure::Tool(crate::render::errors::Tool {
                command: crate::render::errors::Command::CatFile,
                ended: crate::render::errors::Ended::Output,
                tail: Box::new([])
            })
        }),
        "a failed batch closes immediately"
    );
}

#[test]
fn a_task_json_reaches_render_through_declaration() {
    let mut layer = Layer::new();
    let blob =
        crate::render::vocabulary::Blob { kind: crate::render::vocabulary::As::Declaration, ..fixtures::blob(b"{}") };
    layer.read(Box::new([blob]));
    assert_eq!(
        layer.run(&fixtures::batch(b"{}"), Exit::Code(0)),
        RootEvent::Render(Event::Read {
            request: REQUEST,
            oid: fixtures::oid(),
            read: Decoded::Declaration(crate::render::revision::declaration::Declaration {
                at: Position { line: 1, column: 1 },
                types: Box::new([]),
                settings: Box::new([]),
                requires: Box::new([]),
                exports: Box::new([]),
                entries: Box::new([]),
                reads: Box::new([])
            })
        })
    );
}
#[test]
fn a_declaration_config_refuses_answers_refused_with_its_fault() {
    let mut layer = Layer::new();
    let blob = crate::render::vocabulary::Blob {
        kind: crate::render::vocabulary::As::Declaration,
        ..fixtures::blob(b"{\"setting\":{}}")
    };
    layer.read(Box::new([blob]));
    assert_eq!(
        layer.run(&fixtures::batch(b"{\"setting\":{}}"), Exit::Code(0)),
        RootEvent::Render(Event::Read {
            request: REQUEST,
            oid: fixtures::oid(),
            read: Decoded::Refused {
                at: Position { line: 1, column: 2 },
                fault: crate::render::errors::Decode::Declaration(crate::render::errors::Declared::Unknown {
                    of: crate::render::errors::Object::Declaration
                }),
                also: None
            }
        })
    );
}

#[test]
fn a_missing_object_answers_operational_with_its_oid() {
    let mut layer = Layer::new();
    layer.read(Box::new([fixtures::blob(b"{}")]));
    assert_eq!(
        layer.run(b"0123456789abcdef0123456789abcdef01234567 missing\n", Exit::Code(0)),
        RootEvent::Render(Event::Failed {
            request: REQUEST,
            class: crate::render::vocabulary::Class::Operational,
            at: None,
            failure: crate::render::vocabulary::Failure::Tool(crate::render::errors::Tool {
                command: crate::render::errors::Command::CatFile,
                ended: crate::render::errors::Ended::Missing {
                    oid: crate::render::revision::Oid(bytes::copy_of(b"0123456789abcdef0123456789abcdef01234567"))
                },
                tail: Box::new([])
            })
        })
    );
}

#[test]
fn a_read_delivers_each_document_before_finishing_its_child() {
    let mut layer = Layer::new();
    layer.read(Box::new([fixtures::blob(b"{}"), fixtures::blob(b"{}")]));
    let response = fixtures::batch(b"{}");
    let mut writer = skein_lib::Writer::new(response.len().saturating_mul(2));
    writer.put(&response).expect("first response");
    writer.put(&response).expect("second response");
    let mut held = Held::new(writer.finish());
    let mut count = 0_u32;
    for _ in 0_u32..256 {
        if layer.told().is_some() {
            count = count.saturating_add(1);
        }
        let Some(request) = layer.asked() else {
            if layer.told().is_some() {
                count = count.saturating_add(1);
                continue;
            }
            break;
        };
        layer.answer(request, &mut held, Exit::Code(0));
    }
    assert_eq!(count, 2);
}

#[test]
fn a_read_waiting_for_stderr_end_allows_its_completion_through() {
    let mut layer = Layer::new();
    layer.read(Box::new([crate::render::vocabulary::Blob {
        kind: crate::render::vocabulary::As::Declaration,
        ..fixtures::blob(b"{}")
    }]));
    let Some(Asked::Io(io::Request::Spawn { owner, .. })) = layer.asked() else { panic!("spawn") };
    layer.spawned(owner, true, Exit::Code(0));
    let mut held = Held::new(fixtures::batch(b"{}"));
    for _ in 0_u32..64 {
        let Some(request) = layer.asked() else {
            break;
        };
        if let Asked::Io(io::Request::Stream { stream: fixtures::ERROR, .. }) = request {
            continue;
        }
        layer.answer(request, &mut held, Exit::Code(0));
    }
    assert_eq!(layer.told(), None, "stderr still owes its end");
    layer.up(io::Event::Stream { owner: fixtures::ERROR, up: skein_lib::stream::Up::End });
    assert_eq!(
        layer.run(b"", Exit::Code(0)),
        RootEvent::Render(Event::Read {
            request: REQUEST,
            oid: fixtures::oid(),
            read: Decoded::Declaration(crate::render::revision::declaration::Declaration {
                at: Position { line: 1, column: 1 },
                types: Box::new([]),
                settings: Box::new([]),
                requires: Box::new([]),
                exports: Box::new([]),
                entries: Box::new([]),
                reads: Box::new([])
            })
        })
    );
}

#[test]
fn a_templates_blob_reaches_render_compiled_with_its_lines() {
    use crate::render::evaluate::template::{Does, Instruction, Template};
    let mut layer = Layer::new();
    layer.read(Box::new([crate::render::vocabulary::Blob {
        kind: crate::render::vocabulary::As::Template,
        ..fixtures::blob(b"first\nsecond")
    }]));
    assert_eq!(
        layer.run(&fixtures::batch(b"first\nsecond"), Exit::Code(0)),
        RootEvent::Render(Event::Read {
            request: REQUEST,
            oid: fixtures::oid(),
            read: Decoded::Template(Template {
                instructions: Box::new([Instruction { at: 0, does: Does::Text(bytes::copy_of(b"first\nsecond")) }]),
                lines: Box::new([0, 6]),
            }),
        })
    );
}
#[test]
fn a_template_that_does_not_compile_is_refused_at_its_line_and_column() {
    let mut layer = Layer::new();
    layer.read(Box::new([crate::render::vocabulary::Blob {
        kind: crate::render::vocabulary::As::Template,
        ..fixtures::blob(b"Host:\n{{ target.name\n")
    }]));
    assert_eq!(
        layer.run(&fixtures::batch(b"Host:\n{{ target.name\n"), Exit::Code(0)),
        RootEvent::Render(Event::Read {
            request: REQUEST,
            oid: fixtures::oid(),
            read: Decoded::Refused {
                at: Position { line: 2, column: 1 },
                fault: crate::render::errors::Decode::Syntax(crate::render::errors::Syntax::Unclosed),
                also: None,
            },
        })
    );
}

#[test]
fn a_guards_condition_reaches_render_compiled_and_its_message_as_written() {
    use crate::render::{
        evaluate::{Expression, Op, Operation},
        revision::declaration::{Condition, Guard, Text},
    };
    let declaration = fixtures::read_declaration(fixtures::GUARDED);
    assert_eq!(
        declaration.requires.as_ref(),
        &[Guard {
            name: Text { at: Position { line: 1, column: 14 }, text: bytes::copy_of(b"ssh") },
            at: Position { line: 1, column: 20 },
            when: Condition {
                at: Position { line: 1, column: 26 },
                expression: Expression {
                    values: 1,
                    operations: Box::new([Operation { at: 0, op: Op::Boolean(true) }])
                }
            },
            message: Text { at: Position { line: 1, column: 43 }, text: bytes::copy_of(b"keep as written") },
        }]
    );
}
#[test]
fn a_condition_that_does_not_compile_refuses_its_declaration_at_the_string() {
    assert_eq!(
        fixtures::declaration_event(br#"{"requires":{"ssh":{"if":"@","message":"plain"}}}"#),
        RootEvent::Render(Event::Read {
            request: REQUEST,
            oid: fixtures::oid(),
            read: Decoded::Refused {
                at: Position { line: 1, column: 26 },
                fault: crate::render::errors::Decode::Syntax(crate::render::errors::Syntax::Character),
                also: None
            }
        })
    );
}
const TAG_MESSAGES: &[&[u8]] = &[
    br#"{"requires":{"ssh":{"if":"true","message":"{{ x }}"}}}"#,
    br#"{"requires":{"ssh":{"if":"true","message":"{% if %}"}}}"#,
    br#"{"requires":{"ssh":{"if":"true","message":"{# #}"}}}"#,
];
#[test]
fn a_message_holding_a_tag_or_a_comment_is_refused_at_its_string() {
    for (row, body) in TAG_MESSAGES.iter().enumerate() {
        assert_eq!(
            fixtures::declaration_event(body),
            RootEvent::Render(Event::Read {
                request: REQUEST,
                oid: fixtures::oid(),
                read: Decoded::Refused {
                    at: Position { line: 1, column: 43 },
                    fault: crate::render::errors::Decode::Syntax(crate::render::errors::Syntax::Interpolation),
                    also: None
                }
            }),
            "row {row}"
        );
    }
}

mod links;
