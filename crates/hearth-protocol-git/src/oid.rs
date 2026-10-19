//! Git object ids, read exactly as git prints them (domain/render.md, 2.2).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Oid {
    digits: [u8; 64],
    length: u8,
}
impl Oid {
    /// Forty or sixty-four lowercase hexadecimal digits.
    #[must_use]
    pub fn parse(input: &[u8]) -> Option<Self> {
        if input.len() != 40 && input.len() != 64 {
            return None;
        }
        let mut digits = [0; 64];
        for (slot, byte) in digits.iter_mut().zip(input) {
            if !byte.is_ascii_digit() && !(b'a'..=b'f').contains(byte) {
                return None;
            }
            *slot = *byte;
        }
        Some(Self { digits, length: u8::try_from(input.len()).expect("validated id length") })
    }
    /// The id's lowercase hexadecimal bytes.
    #[must_use]
    pub fn hex(&self) -> &[u8] {
        self.digits.get(..usize::from(self.length)).expect("validated id length")
    }
}
