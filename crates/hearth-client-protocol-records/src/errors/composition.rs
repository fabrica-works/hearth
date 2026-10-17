use super::Lines;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Composition {
    Duplicate,
    Long,
}
impl Composition {
    pub(super) fn text(self, lines: &mut Lines, target: Option<&[u8]>) {
        match self {
            Self::Duplicate => {
                lines.put(b"two targets are named ");
                if let Some(target) = target {
                    lines.put(target);
                }
            }
            Self::Long => lines.put(b"a target name past 63 bytes"),
        }
    }
    pub(super) fn name(self) -> &'static [u8] {
        match self {
            Self::Duplicate => b"duplicate",
            Self::Long => b"long",
        }
    }
}
