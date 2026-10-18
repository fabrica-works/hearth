use super::super::{File, Form, Rendered, Repository};
pub const HEARTH: &[File] = &[
    File { path: "tasks/hearth/task.json", body: include_bytes!("hearth.json"), mode: 0o644 },
    File {
        path: "tasks/hearth/tree/etc/systemd/system/hearth-activator.service",
        body: b"[Service]\nExecStart=/usr/local/bin/hearth\n",
        mode: 0o644,
    },
    File { path: "tasks/hearth/tree/usr/local/lib/hearth/askpass", body: b"#!/bin/sh\n", mode: 0o755 },
    File {
        path: "tasks/hearth/tree/etc/hearth/alarms.json.j2",
        body: b"{{ settings.alarms | tojson }}\n",
        mode: 0o644,
    },
];
pub const CONTAINER: File = File {
    path: "hosts/chat/chat-prod.json",
    body: br#"{"placement": {"kind": "container", "on": "rhi-prod"}}"#,
    mode: 0o644,
};
pub const CONTAINER_HEARTH: File = File {
    path: "hosts/chat/chat-prod.json",
    body: br#"{"placement": {"kind": "container", "on": "rhi-prod"}, "hearth": {}}"#,
    mode: 0o644,
};
#[must_use]
pub fn no_hearth() -> Vec<File> {
    let mut files = Vec::new();
    for file in super::super::CONTAINERS {
        if file.path != "tasks/hearth/task.json" && file.path != CONTAINER.path {
            files.push(*file);
        }
    }
    files.push(CONTAINER);
    files
}
#[must_use]
pub fn render_no_hearth(target: &[u8]) -> Rendered {
    let repository = Repository::make("no-hearth", &no_hearth());
    super::super::render(&repository, &[target], Form::Text)
}
#[must_use]
pub fn render_hearth() -> Rendered {
    super::rendered_files(HEARTH)
}
#[must_use]
pub fn render_empty_hearth() -> Rendered {
    let repository = Repository::make("empty-hearth", super::super::TEST_REPOSITORY);
    super::super::render(&repository, &[b"test-min"], Form::Text)
}
#[must_use]
pub fn render_container_hearth() -> Rendered {
    let mut files = no_hearth();
    files.push(CONTAINER_HEARTH);
    let repository = Repository::make("container-hearth", &files);
    super::super::render(&repository, &[b"chat-prod"], Form::Text)
}
