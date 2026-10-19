use hearth_protocol_accounts::{
    File,
    limits::{self, Limits},
};
pub const LIMITS: Limits = limits::SHIPPED;
pub const FILES: [File; 4] = [File::Passwd, File::Group, File::Shadow, File::Gshadow];
fn line(file: File, name: &str) -> String {
    match file {
        File::Passwd => format!("{name}:x:65535:65535:{name}:/home:/bin/sh\n"),
        File::Group => format!("{name}:x:65535:{name}\n"),
        File::Shadow => format!("{name}:!:19000:0:99999:7:::\n"),
        File::Gshadow => format!("{name}:!::{name}\n"),
    }
}
/// Files that fill the limits: the most lines, and the longest names that fit the file.
#[must_use]
pub fn at_the_limits(file: File, limits: &Limits) -> [Vec<u8>; 2] {
    let size = usize::try_from(limits.file).expect("bounded fixture value");
    let lines = usize::try_from(limits.lines).expect("bounded fixture value");
    let width = usize::try_from(limits.name).expect("bounded fixture value");
    let mut most = Vec::new();
    for number in 0..lines {
        let next = line(file, &format!("{number:x}"));
        if most.len() + next.len() > size {
            break;
        }
        most.extend_from_slice(next.as_bytes());
    }
    let mut longest = Vec::new();
    for number in 0..lines {
        let name = format!("{number:04x}{}", "n".repeat(width.saturating_sub(4)));
        let next = line(file, &name);
        if longest.len() + next.len() > size {
            break;
        }
        longest.extend_from_slice(next.as_bytes());
    }
    [most, longest]
}
