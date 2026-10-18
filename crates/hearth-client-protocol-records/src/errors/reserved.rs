//! Closed reserved rules (format.md, 5.11; machine.md, 6.2).
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Reserved {
    pub rule: Rule,
    pub what: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Rule {
    ActivatorKey,
    RootKey,
    RootWait,
    CmdlineLength { bytes: u32 },
    PasswordPreseed,
    TrialName,
    Overlay,
    TmpfilesSubvolume,
    TmpfilesWrite,
    TmpfilesPath,
    ContainerModule,
    ContainerParam,
}
use crate::form::{json::field, text::Lines};
use skein_json::writer::Encoder;
impl Rule {
    const fn name(self) -> &'static [u8] {
        match self {
            Self::ActivatorKey => b"activator_key",
            Self::RootKey => b"root_key",
            Self::RootWait => b"rootwait",
            Self::CmdlineLength { .. } => b"cmdline_length",
            Self::PasswordPreseed => b"password_preseed",
            Self::TrialName => b"trial_name",
            Self::Overlay => b"overlay",
            Self::TmpfilesSubvolume => b"tmpfiles_subvolume",
            Self::TmpfilesWrite => b"tmpfiles_write",
            Self::TmpfilesPath => b"tmpfiles_path",
            Self::ContainerModule => b"container_module",
            Self::ContainerParam => b"container_param",
        }
    }
}
impl Reserved {
    pub(super) fn text(&self, lines: &mut Lines) {
        lines.quote(&self.what);
        lines.put(b" is reserved: ");
        match self.rule {
            Rule::ActivatorKey => lines.put(b"the activator owns this command-line key (machine.md, 6.2)"),
            Rule::RootKey => lines.put(b"it changes how tiny-initramfs finds or starts the root (machine.md, 6.2)"),
            Rule::RootWait => lines.put(b"tiny-initramfs would wait for ever for a missing root (machine.md, 6.2)"),
            Rule::CmdlineLength { bytes } => {
                lines.put(b"the command line is ");
                lines.decimal(u64::from(bytes));
                lines.put(b" bytes, past 2048 with the activator's tokens and NUL (machine.md, 6.2)");
            }
            Rule::PasswordPreseed => {
                lines.put(b"a password preseed leaves its value in /var/cache/debconf/passwords.dat (format.md, 5.11)");
            }
            Rule::TrialName => lines.put(b"hearth owns trial and check units (slots.md, 5.5)"),
            Rule::Overlay => lines.put(b"extensions would merge over hearth's live overlays (format.md, 5.11)"),
            Rule::TmpfilesSubvolume => lines.put(b"a nested subvolume breaks a layer (format.md, 5.11)"),
            Rule::TmpfilesWrite => lines
                .put(b"a tmpfiles write outside /var, /run or /tmp would fail on the read-only root (format.md, 5.11)"),
            Rule::TmpfilesPath => lines.put(b"a tmpfiles path holds no glob or specifier (format.md, 5.11)"),
            Rule::ContainerModule => lines.put(b"a container has no initramfs module (render.md, 4.2)"),
            Rule::ContainerParam => lines.put(b"a container has no boot parameter (render.md, 4.2)"),
        }
    }
    pub(super) fn json(&self, json: &mut Encoder) {
        json.object_start();
        if let Rule::CmdlineLength { bytes } = self.rule {
            json.key(b"bytes");
            json.unsigned(u64::from(bytes));
        }
        field(json, b"rule", self.rule.name());
        field(json, b"what", &self.what);
        json.object_end();
    }
}
