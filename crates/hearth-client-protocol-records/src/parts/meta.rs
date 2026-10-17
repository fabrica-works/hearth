//! The meta part (render.md, 4.4).
use crate::form::text::Lines;
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Meta {
    pub target: Box<[u8]>,
    pub base: Box<[u8]>,
    pub member: Option<Box<[u8]>>,
    pub environment: Box<[u8]>,
    pub placement: Placement,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Placement {
    pub kind: PlacementKind,
    pub firmware: Option<Firmware>,
    pub hardware: Option<Box<[u8]>>,
    pub on: Option<Box<[u8]>>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PlacementKind {
    Metal,
    Vm,
    Container,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Firmware {
    Efi,
    Bios,
}
impl Meta {
    pub(crate) fn text(&self, lines: &mut Lines, column: usize) {
        lines.put(b"target");
        lines.pad_to(column);
        lines.put(&self.target);
        lines.put(b"   base ");
        lines.put(&self.base);
        if let Some(member) = &self.member {
            lines.put(b"   member ");
            lines.put(member);
        }
        lines.put(b"   environment ");
        lines.put(&self.environment);
        lines.put(b"\n");
        lines.put(b"placement");
        lines.pad_to(column);
        lines.put(match self.placement.kind {
            PlacementKind::Metal => b"metal",
            PlacementKind::Vm => b"vm",
            PlacementKind::Container => b"container",
        });
        if let Some(firmware) = self.placement.firmware {
            lines.put(b" ");
            lines.put(match firmware {
                Firmware::Efi => b"efi",
                Firmware::Bios => b"bios",
            });
        }
        if let Some(hardware) = &self.placement.hardware {
            lines.put(b" ");
            lines.put(hardware);
        }
        if let Some(on) = &self.placement.on {
            lines.put(b" on ");
            lines.put(on);
        }
        lines.put(b"\n");
    }
}
