//! Session 30.6's bounded revision and answer inputs.
use hearth_protocol_git::{
    invocation::Exit,
    rev_parse::{self, Error, Resolved},
};
use skein_lib::Rng;
/// render.md 2.2 admits at most 255 revision bytes; ids have the two git widths.
pub const REVISION_LIMIT: usize = 255;
pub const SHA1_DIGITS: usize = 40;
pub const SHA256_DIGITS: usize = 64;
pub const CASES: core::ops::Range<u64> = 0..5_000;
pub const SEED: u64 = 0x0030_6000;
pub const EXITS: &[Exit] = &[Exit::Code(0), Exit::Code(1), Exit::Code(2), Exit::Code(255), Exit::Signal(9)];
const DIGITS: &[u8] = b"0123456789abcdef";
const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._/-";
pub struct Case {
    pub revision: Vec<u8>,
    pub stdout: Vec<u8>,
}
/// Both properties keep the same seed's inputs after their test-shape split.
#[must_use]
pub fn case(rng: &mut Rng) -> Case {
    Case { revision: revision(rng), stdout: answer(rng) }
}
#[must_use]
pub fn revision(rng: &mut Rng) -> Vec<u8> {
    let length = usize::try_from(rng.between(0, (REVISION_LIMIT + 1) as u64)).expect("small revision");
    let mut bytes: Vec<_> =
        (0..length).map(|_| ALPHABET[usize::try_from(rng.below(ALPHABET.len() as u64)).expect("letter")]).collect();
    damage(rng, &mut bytes);
    bytes
}
#[must_use]
pub fn answer(rng: &mut Rng) -> Vec<u8> {
    let length = if rng.chance(500) { SHA1_DIGITS } else { SHA256_DIGITS };
    let mut bytes: Vec<_> =
        (0..length).map(|_| DIGITS[usize::try_from(rng.below(DIGITS.len() as u64)).expect("digit")]).collect();
    bytes.push(b'\n');
    damage(rng, &mut bytes);
    bytes
}
fn damage(rng: &mut Rng, bytes: &mut Vec<u8>) {
    let at = usize::try_from(rng.below(bytes.len() as u64 + 1)).expect("offset");
    match rng.below(4) {
        0 => bytes.truncate(at),
        1 if at < bytes.len() => bytes[at] = u8::try_from(rng.below(u64::from(u8::MAX) + 1)).expect("byte"),
        2 => bytes.push(b'\n'),
        _ => {}
    }
}
/// Normalize only the opaque id's representation, preserving every byte and refusal.
/// # Errors
/// Return the decoder's complete error unchanged.
pub fn decoded(stdout: &[u8], exit: Exit) -> Result<Option<Vec<u8>>, Error> {
    rev_parse::decode(stdout, exit).map(|resolved| match resolved {
        Resolved::Commit(oid) => Some(oid.hex().to_vec()),
        Resolved::Unresolved => None,
    })
}
