//! format.md's nine complete configuration examples.
#[derive(Clone, Copy, Debug)]
pub struct Example {
    pub name: &'static str,
    pub bytes: &'static [u8],
}
#[must_use]
pub fn all() -> &'static [Example] {
    &[
        Example { name: "ion-prod", bytes: include_bytes!("../examples/ion-prod.json") },
        Example { name: "chat-prod", bytes: include_bytes!("../examples/chat-prod.json") },
        Example { name: "stoa", bytes: include_bytes!("../examples/stoa.json") },
        Example { name: "fleet", bytes: include_bytes!("../examples/fleet.json") },
        Example { name: "x1", bytes: include_bytes!("../examples/x1.json") },
        Example { name: "x1-prod", bytes: include_bytes!("../examples/x1-prod.json") },
        Example { name: "mail", bytes: include_bytes!("../examples/mail.json") },
        Example { name: "mail-lab", bytes: include_bytes!("../examples/mail-lab.json") },
        Example { name: "carbon-lab", bytes: include_bytes!("../examples/carbon-lab.json") },
    ]
}

/// A task example from format.md.
#[must_use]
pub fn task(name: &str) -> Example {
    for example in TASKS {
        if example.name == name {
            return *example;
        }
    }
    panic!("unknown task example: {name}");
}
const TASKS: &[Example] = &[
    Example { name: "reads", bytes: include_bytes!("../examples/tasks/reads.json") },
    Example { name: "scripts", bytes: include_bytes!("../examples/tasks/scripts.json") },
    Example { name: "bless", bytes: include_bytes!("../examples/tasks/bless.json") },
];
