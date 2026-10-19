//! Closed reserved rules (protocol/format.md, 5.11; domain/machine.md, 6.2).
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
    ActivatorFile,
    Dropin,
    Accounts,
    SecretPath,
    HearthState,
    PackagesLayer,
    ClassMount,
    DataRoot,
    Runtime,
    LogFile,
    InitramfsConf,
    SecretMode,
    HearthOwner,
    LiveNone,
    NeverLiveUnit,
    NeverLivePath,
    HearthContainer,
    HearthNull,
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
            Self::ActivatorFile => b"activator_file",
            Self::Dropin => b"dropin",
            Self::Accounts => b"accounts",
            Self::SecretPath => b"secret_path",
            Self::HearthState => b"hearth_state",
            Self::PackagesLayer => b"packages_layer",
            Self::ClassMount => b"class_mount",
            Self::DataRoot => b"data_root",
            Self::Runtime => b"runtime",
            Self::LogFile => b"log_file",
            Self::InitramfsConf => b"initramfs_conf",
            Self::SecretMode => b"secret_mode",
            Self::HearthOwner => b"hearth_owner",
            Self::LiveNone => b"live_none",
            Self::NeverLiveUnit => b"never_live_unit",
            Self::NeverLivePath => b"never_live_path",
            Self::HearthContainer => b"hearth_container",
            Self::HearthNull => b"hearth_null",
        }
    }
}
impl Reserved {
    pub(super) fn text(&self, lines: &mut Lines) {
        if self.rule == Rule::SecretPath {
            lines.put(&self.what);
            lines.put(b" is a secret's path; declare it under secrets (protocol/format.md, 9.1)");
            return;
        }
        lines.quote(&self.what);
        lines.put(b" is reserved: ");
        match self.rule {
            Rule::ActivatorKey => lines.put(b"the activator owns this command-line key (domain/machine.md, 6.2)"),
            Rule::RootKey => lines.put(b"it changes how tiny-initramfs finds or starts the root (domain/machine.md, 6.2)"),
            Rule::RootWait => lines.put(b"tiny-initramfs would wait for ever for a missing root (domain/machine.md, 6.2)"),
            Rule::CmdlineLength { bytes } => {
                lines.put(b"the command line is ");
                lines.decimal(u64::from(bytes));
                lines.put(b" bytes, past 2048 with the activator's tokens and NUL (domain/machine.md, 6.2)");
            }
            Rule::PasswordPreseed => {
                lines.put(b"a password preseed leaves its value in /var/cache/debconf/passwords.dat (protocol/format.md, 5.11)");
            }
            Rule::TrialName => lines.put(b"hearth owns trial and check units (domain/slots.md, 5.5)"),
            Rule::Overlay => lines.put(b"extensions would merge over hearth's live overlays (protocol/format.md, 5.11)"),
            Rule::TmpfilesSubvolume => lines.put(b"a nested subvolume breaks a layer (protocol/format.md, 5.11)"),
            Rule::TmpfilesWrite => lines
                .put(b"a tmpfiles write outside /var, /run or /tmp would fail on the read-only root (protocol/format.md, 5.11)"),
            Rule::TmpfilesPath => lines.put(b"a tmpfiles path holds no glob or specifier (protocol/format.md, 5.11)"),
            Rule::ContainerModule => lines.put(b"a container has no initramfs module (domain/render.md, 4.2)"),
            Rule::ContainerParam => lines.put(b"a container has no boot parameter (domain/render.md, 4.2)"),
            Rule::ActivatorFile => lines.put(b"the activator writes this path at prepare (protocol/format.md, 5.11)"),
            Rule::Dropin => lines.put(b"the watchdog's drop-in must sort last (domain/slots.md, 5.5)"),
            Rule::Accounts => lines.put(b"accounts and machine identity are written at prepare (protocol/format.md, 5.11)"),
            Rule::SecretPath => lines.put(b""),
            Rule::HearthState => lines.put(b"hearth owns its state and store (domain/machine.md, 3.4)"),
            Rule::PackagesLayer => lines.put(b"the packages layer owns /usr outside /usr/local (protocol/format.md, 5.5)"),
            Rule::ClassMount => lines.put(b"the class mount hides this path (protocol/format.md, 5.7)"),
            Rule::DataRoot => lines.put(b"a data path must be strictly beneath /srv, /var or /home (protocol/format.md, 5.7)"),
            Rule::Runtime => lines.put(b"runtime, noise and history are not task files (protocol/format.md, 5.11)"),
            Rule::LogFile => lines.put(b"a file beneath /var/log must be empty (protocol/format.md, 5.11)"),
            Rule::InitramfsConf => {
                lines.put(b"the packages part writes tiny-initramfs configuration (domain/machine.md, 7.1)");
            }
            Rule::SecretMode => {
                lines.put(b"a secret or identity mode gives others nothing and has no special bits (protocol/format.md, 9.1)");
            }
            Rule::HearthOwner => {
                lines.put(b"hearth must not be able to read this secret or identity file (protocol/format.md, 9.1)");
            }
            Rule::LiveNone => lines.put(b"action none names no unit (protocol/format.md, 5.14)"),
            Rule::NeverLiveUnit => lines.put(b"this core service must not restart live (protocol/format.md, 5.14)"),
            Rule::NeverLivePath => lines.put(b"this path must not change live (protocol/format.md, 5.14)"),
            Rule::HearthContainer => lines.put(b"a container has no hearth task (protocol/format.md, 5.12)"),
            Rule::HearthNull => lines.put(b"the hearth task cannot be removed (protocol/format.md, 5.12)"),
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
