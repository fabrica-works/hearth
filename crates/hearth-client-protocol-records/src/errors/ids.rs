//! Missing fixed ids name the concrete owned entry (render.md, 7.2).
use crate::form::{json::field, text::Lines};
use alloc::boxed::Box;
use skein_json::writer::Encoder;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Ids {
    pub owner: Box<[u8]>,
    pub of: Owns,
    pub entry: Box<[u8]>,
    pub why: Why,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Owns {
    Data,
    Secret,
    Identity,
    Download,
    User,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Why {
    NoUser,
    NoGroup,
    NoUid,
    NoGid,
    OneId,
    NotPackageUser,
    NotPackageGroup,
}
impl Owns {
    const fn name(self) -> &'static [u8] {
        match self {
            Self::Data => b"data",
            Self::Secret => b"secret",
            Self::Identity => b"identity",
            Self::Download => b"download",
            Self::User => b"user",
        }
    }
}
impl Why {
    const fn name(self) -> &'static [u8] {
        match self {
            Self::NoUser => b"no_user",
            Self::NoGroup => b"no_group",
            Self::NoUid => b"no_uid",
            Self::NoGid => b"no_gid",
            Self::OneId => b"one_id",
            Self::NotPackageUser => b"not_package_user",
            Self::NotPackageGroup => b"not_package_group",
        }
    }
}
impl Ids {
    pub(super) fn text(&self, lines: &mut Lines) {
        lines.put(&self.owner);
        lines.put(b" owns ");
        lines.put(&self.entry);
        lines.put(b" and has no fixed id: ");
        match self.why {
            Why::NoUser | Why::NoGroup => {
                lines.put(if self.why == Why::NoUser { b"no user " } else { b"no group " });
                lines.put(&self.owner);
                lines.put(b" is declared");
            }
            Why::NoUid => lines.put(b"the user has no uid"),
            Why::NoGid => lines.put(b"the group has no gid"),
            Why::OneId => lines.put(b"a package user needs both uid and gid or neither"),
            Why::NotPackageUser => lines.put(b"a download needs a package user"),
            Why::NotPackageGroup => lines.put(b"a download needs a package group"),
        }
    }
    pub(super) fn json(&self, json: &mut Encoder) {
        json.object_start();
        field(json, b"entry", &self.entry);
        field(json, b"of", self.of.name());
        field(json, b"owner", &self.owner);
        field(json, b"why", self.why.name());
        json.object_end();
    }
}
