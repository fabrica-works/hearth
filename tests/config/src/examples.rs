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
    Example { name: "users-accounts", bytes: include_bytes!("../examples/tasks/users-accounts.json") },
    Example { name: "ssh", bytes: include_bytes!("../examples/tasks/ssh.json") },
    Example { name: "mail-accounts", bytes: include_bytes!("../examples/tasks/mail-accounts.json") },
    Example { name: "core-journal", bytes: include_bytes!("../examples/tasks/core-journal.json") },
    Example { name: "wifi", bytes: include_bytes!("../examples/tasks/wifi.json") },
    Example { name: "mail-data", bytes: include_bytes!("../examples/tasks/mail-data.json") },
    Example { name: "firmware", bytes: include_bytes!("../examples/tasks/firmware.json") },
    Example { name: "core-gettys", bytes: include_bytes!("../examples/tasks/core-gettys.json") },
    Example { name: "mail-live", bytes: include_bytes!("../examples/tasks/mail-live.json") },
    Example { name: "stoa", bytes: include_bytes!("../examples/tasks/stoa.json") },
    Example { name: "ripgrep", bytes: include_bytes!("../examples/tasks/ripgrep.json") },
    Example { name: "dkim", bytes: include_bytes!("../examples/tasks/dkim.json") },
    Example { name: "ssh-identity", bytes: include_bytes!("../examples/tasks/ssh-identity.json") },
    Example { name: "software", bytes: include_bytes!("../examples/tasks/software.json") },
    Example { name: "artifact-stoa", bytes: include_bytes!("../examples/tasks/artifact-stoa.json") },
    Example { name: "checks-ssh", bytes: include_bytes!("../examples/tasks/checks-ssh.json") },
];
