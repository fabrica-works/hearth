use crate::{
    entries::{Expansion, Fielded, Made},
    errors::Error,
    parts::software,
    revision::declaration::{Map, Name},
};
pub const MAX_DOWNLOAD_MODE: i64 = 0o777;
pub const MIN_STRIP: i64 = 0;
pub const MAX_STRIP: i64 = 255;
pub const TAR_WITH_NO_STRIP: software::How = software::How::Tar { strip: 0 };
pub const TAR_WITH_MAX_STRIP: software::How = software::How::Tar { strip: 255 };
pub const ABC_SHA256_HEX: &[u8] = b"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
pub fn download_fields(url: &[u8], unpack: &[u8], path: &[u8]) -> [Made; 4] {
    [
        super::kinds::text(Name::Url, url),
        super::kinds::text(Name::Sha256, ABC_SHA256_HEX),
        super::kinds::text(Name::Unpack, unpack),
        super::kinds::text(Name::Path, path),
    ]
}
pub fn download_entry(overrides: &[Made]) -> Expansion {
    let mut fields = skein_lib::List::with_capacity(12);
    for field in download_fields(b"https://a.example/tool", b"file", b"/opt/tool").iter().chain(overrides) {
        // The last value replaces the field, just as a merged declaration does.
        let mut replaced = false;
        for index in 0..fields.len() {
            let present: &mut Made = fields.get_mut(index).expect("field");
            if present.name == field.name {
                *present = field.clone();
                replaced = true;
            }
        }
        if !replaced {
            fields.push(field.clone()).expect("field count");
        }
    }
    let mut expansion = super::secret_entry(Map::Software, b"tool", &[]);
    for task in &mut expansion.tasks {
        for group in &mut task.groups {
            for entry in &mut group.entries {
                entry.fields = fields.as_slice().into();
            }
        }
    }
    expansion
}
pub fn strip(value: i64) -> Made {
    Made { name: Name::Strip, at: super::kinds::FIELD_AT, value: Fielded::Int(value) }
}
pub fn software(overrides: &[Made]) -> software::Software {
    software::build(&download_entry(overrides), Box::new([])).expect("download")
}
pub fn software_fault(overrides: &[Made]) -> Error {
    super::only(&software::build(&download_entry(overrides), Box::new([])).expect_err("invalid download")).clone()
}
pub fn mirrored(url: &[u8]) -> software::Software {
    software(&[super::kinds::text(Name::Url, url)])
}

pub const SOFTWARE_SCRIPT_DIGEST: hearth_lib::encode::Digest = hearth_lib::encode::Digest([
    210, 169, 199, 48, 47, 206, 113, 126, 81, 240, 47, 196, 89, 11, 64, 0, 4, 219, 129, 122, 115, 84, 203, 168, 206,
    130, 181, 42, 111, 255, 12, 139,
]);
pub fn download_mode(mode: i64) -> Made {
    let mode = u16::try_from(mode).expect("fixture mode");
    let mut digits = [b'0'; 4];
    for (digit, shift) in digits.iter_mut().zip([9_u32, 6, 3, 0]) {
        *digit = b'0'.saturating_add(u8::try_from((mode >> shift) & 7).expect("octal digit"));
    }
    super::kinds::text(Name::Mode, &digits)
}

pub fn missing_download_field(name: Name) -> Error {
    let mut expansion = download_entry(&[]);
    for task in &mut expansion.tasks {
        for group in &mut task.groups {
            for entry in &mut group.entries {
                let mut fields =
                    skein_lib::List::with_capacity(u32::try_from(entry.fields.len()).expect("field count"));
                for field in &entry.fields {
                    if field.name != name {
                        fields.push(field.clone()).expect("field count");
                    }
                }
                entry.fields = fields.into_boxed();
            }
        }
    }
    super::only(&software::build(&expansion, Box::new([])).expect_err("missing field")).clone()
}

pub fn empty_software_part() -> crate::parts::Part {
    let software =
        software::build(&super::concrete_expansion(Map::Software, &[]), Box::new([])).expect("empty software");
    let digest = software::digest(&software);
    crate::parts::Part { digest, record: crate::parts::PartRecord::Software(software) }
}
pub fn empty_software_spec() -> crate::parts::Spec {
    super::spec(b"x1-prod", &[empty_software_part()])
}
