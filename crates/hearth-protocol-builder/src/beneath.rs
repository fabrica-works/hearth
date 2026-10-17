//! Operations under a directory held by the file layer.
use alloc::boxed::Box;
use skein_io::{self as io, kernel::Fd};
use skein_lib::Token;
#[derive(PartialEq, Eq, Debug)]
pub enum Asked {
    Io(io::Request),
    File(io::file::Request),
    Beneath(Beneath),
}
/// File operations whose directory is an io-owned token (io.md, 9).
#[derive(PartialEq, Eq, Debug)]
pub enum Beneath {
    OpenDirectory { owner: Token, root: Token, name: Box<[u8]>, no_follow: bool },
    Rename { owner: Token, from_root: Token, from: Box<[u8]>, to_root: Fd, to: Box<[u8]> },
    Remove { owner: Token, root: Token, name: Box<[u8]>, directory: bool },
}
